use tree_sitter::Parser;
use tree_sitter_xy_build;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    KnownIdent(String),
    UnknownIdent(String),
    Block(Vec<Entry>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    KnownIdent(String),
    UnknownIdent(String),
}
impl Key {
    pub fn is_empty(&self) -> bool {
        match self {
            Key::KnownIdent(s) | Key::UnknownIdent(s) => s.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: Key,
    pub value: Value,
    /// 0-based line of the key in the source.
    pub key_row: usize,
    /// 0-based column of the key in the source.
    pub key_col: usize,
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

    parse_tree(tree.root_node(), source)
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

fn parse_tree(node: tree_sitter::Node, source: &str) -> Result<ConfigFile, Vec<ParseError>> {
    let mut entries = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "entry" {
                entries.push(parse_entry(child, source)?);
            }
        }
    }
    Ok(ConfigFile { entries })
}

fn parse_entry(node: tree_sitter::Node, source: &str) -> Result<Entry, Vec<ParseError>> {
    let mut key: Option<Key> = None;
    let mut value: Option<Value> = None;
    let mut key_row = 0;
    let mut key_col = 0;

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                // I know this could theoretically override itself, but this should be disallowed by tree-sitter (hopefully!)
                "known_ident" | "unknown_ident" => {
                    let pos = child.start_position();
                    key_row = pos.row;
                    key_col = pos.column;
                    if key.as_ref().is_none_or(|k| k.is_empty()) {
                        if child.kind() == "known_ident" {
                            key = Some(Key::KnownIdent(child_text(child, source)));
                        } else {
                            key = Some(Key::UnknownIdent(child_text(child, source)));
                        }
                    } else {
                        if child.kind() == "known_ident" {
                            value = Some(Value::KnownIdent(child_text(child, source)));
                        } else {
                            value = Some(Value::UnknownIdent(child_text(child, source)));
                        }
                    }
                }
                "entry" => {
                    let mut block_entries = Vec::new();
                    collect_block_entries(child, source, &mut block_entries)?;
                    value = Some(Value::Block(block_entries));
                }
                _ => {}
            }
        }
    }

    if key.is_none() || value.is_none() {
        return Err(vec![ParseError {
            message: "entry must have a key and a value".into(),
            row: key_row,
            column: key_col,
        }]);
    }
    Ok(Entry {
        key: key.unwrap(),
        value: value.unwrap(),
        key_row,
        key_col,
    })
}

fn collect_block_entries(
    node: tree_sitter::Node,
    source: &str,
    entries: &mut Vec<Entry>,
) -> Result<(), Vec<ParseError>> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "entry" {
                entries.push(parse_entry(child, source)?);
            }
        }
    }
    Ok(())
}

fn child_text(node: tree_sitter::Node, source: &str) -> String {
    source[node.start_byte()..node.end_byte()].to_string()
}
