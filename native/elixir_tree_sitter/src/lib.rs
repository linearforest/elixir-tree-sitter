use tree_sitter::Parser;

mod atoms {
    rustler::atoms! {
        id
    }
}

#[derive(rustler::NifTaggedEnum)]
enum Language {
    Javascript,
    Html,
    Css,
}

#[rustler::nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[derive(Debug, rustler::NifStruct)]
#[module = "TreeSitter.Node"]
pub struct TSNode {
    pub id: usize,
    pub kind: String,
    pub range: TSRange,
    pub children: Vec<TSNode>,
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

        let range = cursor.node().range();

        let children = vec![];

        let node = TSNode {
            id: node.id(),
            kind: node.kind().to_string(),
            range: TSRange {
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
            },
            children,
        };

        return node;
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

#[rustler::nif]
fn parse(corpus: String, language: Language) -> TSNode {
    let mut parser = Parser::new();

    let lang = match language {
        Language::Javascript => tree_sitter_javascript::language(),
        Language::Html => tree_sitter_html::language(),
        Language::Css => tree_sitter_css::language(),
    };

    parser
        .set_language(lang)
        .expect("Error loading Rust grammar");

    let result = parser.parse(&corpus, None).unwrap();

    let node = result.root_node();

    print_cursor(&corpus, &mut node.walk(), 0);

    return TSNode::from(node);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let corpus = "let a = 1;".to_string();

        let mut parser = Parser::new();

        parser
            .set_language(tree_sitter_javascript::language())
            .expect("Error loading Rust grammar");

        let result = parser.parse(&corpus, None).unwrap();

        let node = result.root_node();

        print_cursor(&corpus, &mut node.walk(), 0);

        assert!(false);
    }
}

rustler::init!("Elixir.TreeSitter", [add, parse]);
