use tree_sitter::Parser;
use tree_sitter_xy_build;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Ident(String),
    Block(Vec<Entry>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigFile {
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub row: usize,
    pub column: usize,
}

pub fn parse(source: &str) -> Result<ConfigFile, Vec<ParseError>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_xy_build::LANGUAGE.into())
        .expect("failed to set XY Build grammar");

    let tree = parser.parse(source, None).ok_or_else(|| {
        vec![ParseError {
            message: "failed to parse source".into(),
            row: 0,
            column: 0,
        }]
    })?;

    if tree.root_node().has_error() {
        let mut errors = Vec::new();
        collect_errors(tree.root_node(), source, &mut errors);
        return Err(errors);
    }

    Ok(parse_tree(tree.root_node(), source))
}

fn collect_errors(node: tree_sitter::Node, source: &str, errors: &mut Vec<ParseError>) {
    if node.is_error() || (node.is_missing() && !node.is_named()) {
        let start = node.start_position();
        errors.push(ParseError {
            message: format!(
                "unexpected syntax at '{}'",
                &source[node.start_byte()..node.end_byte()]
            ),
            row: start.row,
            column: start.column,
        });
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_errors(child, source, errors);
        }
    }
}

fn parse_tree(node: tree_sitter::Node, source: &str) -> ConfigFile {
    let mut entries = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "entry" {
                entries.push(parse_entry(child, source));
            }
        }
    }
    ConfigFile { entries }
}

fn parse_entry(node: tree_sitter::Node, source: &str) -> Entry {
    let mut key = String::new();
    let mut value = Value::Ident(String::new());

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "known_ident" | "unknown_ident" => {
                    if key.is_empty() {
                        key = child_text(child, source);
                    } else {
                        value = Value::Ident(child_text(child, source));
                    }
                }
                "entry" => {
                    let mut block_entries = Vec::new();
                    collect_block_entries(child, source, &mut block_entries);
                    value = Value::Block(block_entries);
                }
                _ => {}
            }
        }
    }

    Entry { key, value }
}

fn collect_block_entries(node: tree_sitter::Node, source: &str, entries: &mut Vec<Entry>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "entry" {
                entries.push(parse_entry(child, source));
            }
        }
    }
}

fn child_text(node: tree_sitter::Node, source: &str) -> String {
    source[node.start_byte()..node.end_byte()].to_string()
}
