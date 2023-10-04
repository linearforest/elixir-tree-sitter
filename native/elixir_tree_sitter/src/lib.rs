use tree_sitter::Parser;

mod atoms {
    rustler::atoms! {}
}

#[derive(Debug, rustler::NifTaggedEnum)]
enum Language {
    Javascript,
    Html,
    Css,
}

#[derive(Debug, rustler::NifTaggedEnum)]
enum ParseError {
    ParseError,
    LanguageError,
}

#[derive(Debug, rustler::NifTaggedEnum)]
enum NodeType {
    Named,
    Error,
    Extra,
    Missing,
    Anonymous,
}

#[derive(Debug, rustler::NifStruct)]
#[module = "TreeSitter.Token"]
pub struct Token {
    kind: String,
    value: String,
    node_type: NodeType,
}

#[derive(Debug, rustler::NifStruct)]
#[module = "TreeSitter.Node"]
pub struct TSNode {
    id: usize,
    kind: String,
    range: TSRange,
    children: Vec<TSNode>,
    is_named: bool,
    is_error: bool,
    is_extra: bool,
    is_missing: bool,
}

#[derive(Debug, rustler::NifStruct)]
#[module = "TreeSitter.Range"]
pub struct TSRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_point: TSPoint,
    pub end_point: TSPoint,
}

#[derive(Debug, rustler::NifStruct)]
#[module = "TreeSitter.Point"]
pub struct TSPoint {
    row: usize,
    column: usize,
}

impl TSNode {
    // Convert a tree-sitter node into a our TSNode struct, which can be encoded
    // and sent to Elixir.
    fn from(node: tree_sitter::Node) -> TSNode {
        let mut cursor = node.walk();

        let children = node
            .children(&mut cursor)
            .map(|child| TSNode::from(child))
            .collect::<Vec<_>>();

        let range = cursor.node().range();

        let range = TSRange {
            start_byte: range.start_byte,
            end_byte: range.end_byte,
            start_point: TSPoint {
                row: range.start_point.row,
                column: range.start_point.column,
            },
            end_point: TSPoint {
                row: range.end_point.row,
                column: range.end_point.column,
            },
        };

        let node = TSNode {
            id: node.id(),
            kind: node.kind().to_string(),
            is_named: node.is_named(),
            is_error: node.is_error(),
            is_extra: node.is_extra(),
            is_missing: node.is_missing(),
            range,
            children,
        };

        return node;
    }
}

fn collect_tokens(source: &[u8], cursor: &mut tree_sitter::TreeCursor) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let node = cursor.node();

        if node.child_count() == 0 {
            let text = node.utf8_text(&source).expect("Error getting text");

            let node_type = if node.is_error() {
                NodeType::Error
            } else if node.is_extra() {
                NodeType::Extra
            } else if node.is_missing() {
                NodeType::Missing
            } else if node.is_named() {
                NodeType::Named
            } else {
                NodeType::Anonymous
            };

            let value = Token {
                kind: node.kind().to_owned(),
                value: text.to_owned(),
                node_type,
            };

            tokens.push(value);
        }

        if cursor.goto_first_child() {
            let mut child_tokens = collect_tokens(source, cursor);
            tokens.append(&mut child_tokens);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break tokens;
        }
    }
}

fn print_cursor(src: &str, cursor: &mut tree_sitter::TreeCursor, depth: usize) {
    loop {
        let node = cursor.node();
        node.end_position();

        let formatted_node = format!(
            "{} {} - {}",
            node.kind().replace('\n', "\\n"),
            node.start_position(),
            node.end_position()
        );

        if node.child_count() == 0 {
            let node_src = &src[node.start_byte()..node.end_byte()];
            println!("{}{} {:?}", "  ".repeat(depth), formatted_node, node_src);
        } else {
            println!("{}{}", "  ".repeat(depth), formatted_node,);
        }

        if cursor.goto_first_child() {
            print_cursor(src, cursor, depth + 1);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

fn get_language(language: Language) -> tree_sitter::Language {
    match language {
        Language::Javascript => tree_sitter_javascript::language(),
        Language::Html => tree_sitter_html::language(),
        Language::Css => tree_sitter_css::language(),
    }
}

#[rustler::nif]
fn parse(corpus: String, language: Language) -> Result<TSNode, ParseError> {
    let mut parser = Parser::new();
    let lang = get_language(language);

    parser
        .set_language(lang)
        .map_err(|_| ParseError::LanguageError)
        .and_then(|_| parser.parse(&corpus, None).ok_or(ParseError::ParseError))
        .map(|tree| TSNode::from(tree.root_node()))
}

#[rustler::nif]
fn lex(corpus: String, language: Language) -> Result<Vec<Token>, ParseError> {
    let mut parser = Parser::new();
    let lang = get_language(language);

    parser
        .set_language(lang)
        .map_err(|_| ParseError::LanguageError)
        .and_then(|_| parser.parse(&corpus, None).ok_or(ParseError::ParseError))
        .map(|tree| collect_tokens(&corpus.as_bytes(), &mut tree.root_node().walk()))
}

#[rustler::nif]
fn to_sexp(corpus: String, language: Language) -> Result<String, ParseError> {
    let mut parser = Parser::new();
    let lang = get_language(language);

    parser
        .set_language(lang)
        .map_err(|_| ParseError::LanguageError)
        .and_then(|_| parser.parse(&corpus, None).ok_or(ParseError::ParseError))
        .map(|tree| tree.root_node().to_sexp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(true)
    }
}

rustler::init!("Elixir.TreeSitter", [parse, to_sexp, lex]);
