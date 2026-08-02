use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    docs: Mutex<HashMap<Url, String>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            docs: Mutex::new(HashMap::new()),
        }
    }

    fn hover_at_position(&self, uri: &Url, position: &Position) -> Option<(String, String)> {
        let docs = self.docs.lock().ok()?;
        let content = docs.get(uri)?;
        let line = content.lines().nth(position.line as usize)?;

        let (word, _) = extract_word(line, position.character as usize)?;
        let field = find_field(Config::schema(), &word)?;
        Some((word.to_string(), field.doc.to_string()))
    }

    fn validate(&self, uri: &Url) -> Vec<Diagnostic> {
        let content = match self.docs.lock() {
            Ok(docs) => match docs.get(uri) {
                Some(c) => c.clone(),
                None => return vec![],
            },
            Err(_) => return vec![],
        };

        let config = match xy_build_parser::parse(&content) {
            Ok(c) => c,
            Err(errors) => {
                return errors
                    .into_iter()
                    .map(|e| Diagnostic {
                        range: Range {
                            start: Position {
                                line: e.row as u32,
                                character: e.column as u32,
                            },
                            end: Position {
                                line: e.row as u32,
                                character: e.column as u32 + 1,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("xy-build".to_string()),
                        message: e.message,
                        ..Default::default()
                    })
                    .collect();
            }
        };

        let mut diags = Vec::new();
        validate_entries(&config.entries, Config::schema(), &mut diags);
        diags
    }
}

fn extract_word(line: &str, col: usize) -> Option<(&str, usize)> {
    let start = line[..col]
        .rfind(|c: char| c.is_whitespace() || c == ':' || c == ';' || c == '"')
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = line[col..]
        .find(|c: char| c.is_whitespace() || c == ':' || c == ';' || c == '"')
        .map(|i| col + i)
        .unwrap_or(line.len());
    if start >= end {
        return None;
    }
    Some((&line[start..end], start))
}

fn find_field<'a>(schema: &'a ObjSchema, name: &str) -> Option<&'a Field> {
    schema.fields.iter().find(|f| f.name == name)
}

fn validate_entries(
    entries: &[xy_build_parser::Entry],
    schema: &ObjSchema,
    diags: &mut Vec<Diagnostic>,
) {
    for entry in entries {
        let known = schema.fields.iter().find(|f| f.name == entry.key);
        match known {
            Some(field) => validate_value(&entry.value, field.kind, entry, diags),
            None => match schema.rest {
                Some(rest) => validate_rest_value(&entry.value, rest, entry, diags),
                None => diags.push(diagnostic(
                    entry.key_row,
                    entry.key_col,
                    format!("unknown option '{}'", entry.key),
                    DiagnosticSeverity::WARNING,
                )),
            },
        }
    }
}

fn validate_value(
    value: &xy_build_parser::Value,
    kind: FieldKind,
    entry: &xy_build_parser::Entry,
    diags: &mut Vec<Diagnostic>,
) {
    match (value, kind) {
        (xy_build_parser::Value::Ident(_), FieldKind::Str) => {}
        (xy_build_parser::Value::Ident(v), FieldKind::Enum(variants)) => {
            if !variants.contains(&v.as_str()) {
                diags.push(diagnostic(
                    entry.key_row,
                    entry.key_col,
                    format!(
                        "invalid value '{}', expected one of [{}]",
                        v,
                        variants.join(", ")
                    ),
                    DiagnosticSeverity::ERROR,
                ));
            }
        }
        (xy_build_parser::Value::Ident(_), FieldKind::Object(_)) => {
            diags.push(diagnostic(
                entry.key_row,
                entry.key_col,
                "expected a block (indented sub-entries)".to_string(),
                DiagnosticSeverity::ERROR,
            ));
        }
        (xy_build_parser::Value::Block(_), FieldKind::Str | FieldKind::Enum(_)) => {
            diags.push(diagnostic(
                entry.key_row,
                entry.key_col,
                "expected a simple value, got a block".to_string(),
                DiagnosticSeverity::ERROR,
            ));
        }
        (xy_build_parser::Value::Block(children), FieldKind::Object(sub)) => {
            validate_entries(children, sub, diags);
        }
    }
}

fn validate_rest_value(
    value: &xy_build_parser::Value,
    rest: RestKind,
    entry: &xy_build_parser::Entry,
    diags: &mut Vec<Diagnostic>,
) {
    match (value, rest) {
        (xy_build_parser::Value::Ident(_), RestKind::Str) => {}
        (xy_build_parser::Value::Ident(_), RestKind::Object(_)) => {
            diags.push(diagnostic(
                entry.key_row,
                entry.key_col,
                "expected a block, got a simple value".to_string(),
                DiagnosticSeverity::ERROR,
            ));
        }
        (xy_build_parser::Value::Block(_), RestKind::Str) => {
            diags.push(diagnostic(
                entry.key_row,
                entry.key_col,
                "expected a simple value, got a block".to_string(),
                DiagnosticSeverity::ERROR,
            ));
        }
        (xy_build_parser::Value::Block(children), RestKind::Object(sub)) => {
            validate_entries(children, sub, diags);
        }
    }
}

fn diagnostic(row: usize, col: usize, message: String, severity: DiagnosticSeverity) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: row as u32,
                character: col as u32,
            },
            end: Position {
                line: row as u32,
                character: col as u32 + 1,
            },
        },
        severity: Some(severity),
        source: Some("xy-build".to_string()),
        message,
        ..Default::default()
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
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
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
            docs.insert(uri.clone(), text);
        }
        let diags = self.validate(&uri);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            if let Ok(mut docs) = self.docs.lock() {
                docs.insert(uri.clone(), change.text);
            }
        }
        let diags = self.validate(&uri);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Ok(mut docs) = self.docs.lock() {
            docs.remove(&params.text_document.uri);
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params;
        if let Some((word, docs)) = self.hover_at_position(&pos.text_document.uri, &pos.position) {
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

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut items: Vec<CompletionItem> = Config::schema()
            .fields
            .iter()
            .map(|field| CompletionItem {
                label: field.name.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(field.doc.to_string()),
                insert_text: Some(field.name.to_string()),
                ..Default::default()
            })
            .collect();

        items.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
