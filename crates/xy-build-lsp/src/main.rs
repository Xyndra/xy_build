use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use xy_build_options as options;

struct Backend {
    client: Client,
    docs: Mutex<HashMap<Url, String>>,
    known_options: HashMap<String, String>,
}

impl Backend {
    fn new(client: Client) -> Self {
        let known_options = options::all_options()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self {
            client,
            docs: Mutex::new(HashMap::new()),
            known_options,
        }
    }

    fn word_at_position(&self, uri: &Url, position: &Position) -> Option<(String, String)> {
        let docs = self.docs.lock().ok()?;
        let content = docs.get(uri)?;
        let line = content.lines().nth(position.line as usize)?;

        let start = line[..position.character as usize]
            .rfind(|c: char| c.is_whitespace() || c == ':' || c == ';' || c == '"')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[position.character as usize..]
            .find(|c: char| c.is_whitespace() || c == ':' || c == ';' || c == '"')
            .map(|i| position.character as usize + i)
            .unwrap_or(line.len());

        if start >= end {
            return None;
        }

        let word = &line[start..end];
        self.known_options
            .get(word)
            .map(|docs| (word.to_string(), docs.clone()))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["\n".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "xy-build-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "xy-build-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        if let Ok(mut docs) = self.docs.lock() {
            docs.insert(uri, text);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            if let Ok(mut docs) = self.docs.lock() {
                docs.insert(uri, change.text);
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(mut docs) = self.docs.lock() {
            docs.remove(&params.text_document.uri);
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params;
        let uri = &pos.text_document.uri;
        let position = pos.position;

        if let Some((word, docs)) = self.word_at_position(uri, &position) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}**\n\n{}", word, docs),
                }),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut items: Vec<CompletionItem> = self
            .known_options
            .iter()
            .map(|(key, docs)| CompletionItem {
                label: key.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(docs.clone()),
                insert_text: Some(key.clone()),
                ..Default::default()
            })
            .collect();

        items.sort_by(|a, b| a.label.cmp(&b.label));

        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
