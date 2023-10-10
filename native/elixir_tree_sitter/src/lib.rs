#![feature(assert_matches)]

use std::collections::HashMap;

use tree_sitter::{Parser, Query, QueryCursor};

extern "C" {
    fn tree_sitter_liquid() -> tree_sitter::Language;
}

extern "C" {
    fn tree_sitter_liquid_template() -> tree_sitter::Language;
}

mod atoms {
    rustler::atoms! {}
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, rustler::NifTaggedEnum)]
enum Language {
    Javascript,
    Html,
    Css,
    Liquid,
    LiquidTemplate,
}

#[derive(Debug, PartialEq, rustler::NifTaggedEnum)]
enum ParseError {
    ParseError,
    LanguageError,
    QueryError { message: String },
}

#[derive(Debug, PartialEq, Eq, rustler::NifTaggedEnum)]
enum NodeType {
    Named,
    Error,
    Extra,
    Missing,
    Anonymous,
}

#[derive(Debug, PartialEq, Eq, rustler::NifStruct)]
#[module = "TreeSitter.Token"]
pub struct Token {
    kind: String,
    value: String,
    node_type: NodeType,
}

#[derive(Debug, PartialEq, Eq, rustler::NifStruct)]
#[module = "TreeSitter.Node"]
pub struct TSNode {
    id: usize,
    kind: String,
    range: TSRange,
    children: Vec<TSNode>,
    node_type: NodeType,
    value: Option<String>,
}

#[derive(Debug, PartialEq, Eq, rustler::NifStruct)]
#[module = "TreeSitter.Range"]
pub struct TSRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_point: TSPoint,
    pub end_point: TSPoint,
}

#[derive(Debug, PartialEq, Eq, rustler::NifStruct)]
#[module = "TreeSitter.Point"]
pub struct TSPoint {
    row: usize,
    column: usize,
}

impl TSNode {
    // Convert a tree-sitter node into a our TSNode struct, which can be encoded
    // and sent to Elixir.
    fn from(node: tree_sitter::Node, corpus: &[u8]) -> TSNode {
        let mut cursor = node.walk();

        let children = node
            .children(&mut cursor)
            .map(|child| TSNode::from(child, corpus))
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

        let value = if node.child_count() == 0 {
            node.utf8_text(&corpus).ok().map(|s| s.to_owned())
        } else {
            None
        };

        let node = TSNode {
            id: node.id(),
            kind: node.kind().to_string(),
            node_type: node_type(node),
            range,
            value,
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

            let node_type = node_type(node);

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

fn node_type(node: tree_sitter::Node<'_>) -> NodeType {
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
    node_type
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

fn get_language(language: &Language) -> tree_sitter::Language {
    match language {
        Language::Javascript => tree_sitter_javascript::language(),
        Language::Html => tree_sitter_html::language(),
        Language::Css => tree_sitter_css::language(),
        Language::Liquid => unsafe { tree_sitter_liquid() },
        Language::LiquidTemplate => unsafe { tree_sitter_liquid_template() },
    }
}

fn get_language_from_string(language_string: &str) -> Language {
    match language_string {
        "html" => Language::Html,
        "liquid" => Language::Liquid,
        _ => panic!("Unknown injection language"),
    }
}

fn get_parser(language: Language) -> Result<Parser, ParseError> {
    let mut parser = Parser::new();
    let lang = get_language(&language);

    parser
        .set_language(lang)
        .map_err(|_| ParseError::LanguageError)
        .map(|_| parser)
}

fn do_parse(corpus: String, language: Language) -> Result<TSNode, ParseError> {
    let mut parser = Parser::new();
    let lang = get_language(&language);

    parser
        .set_language(lang)
        .map_err(|_| ParseError::LanguageError)
        .and_then(|_| parser.parse(&corpus, None).ok_or(ParseError::ParseError))
        .map(|tree| TSNode::from(tree.root_node(), corpus.as_bytes()))
}

fn do_to_tokens(corpus: String, language: Language) -> Result<Vec<Token>, ParseError> {
    get_parser(language)
        .and_then(|mut parser| parser.parse(&corpus, None).ok_or(ParseError::ParseError))
        .map(|tree| collect_tokens(&corpus.as_bytes(), &mut tree.root_node().walk()))
}

#[rustler::nif]
fn parse(corpus: String, language: Language) -> Result<TSNode, ParseError> {
    do_parse(corpus, language)
}

#[rustler::nif]
fn to_tokens(corpus: String, language: Language) -> Result<Vec<Token>, ParseError> {
    do_to_tokens(corpus, language)
}

#[rustler::nif]
fn to_sexp(corpus: String, language: Language) -> Result<String, ParseError> {
    get_parser(language)
        .and_then(|mut parser| parser.parse(&corpus, None).ok_or(ParseError::ParseError))
        .map(|tree| tree.root_node().to_sexp())
}

// TODO: replace with a more generic approach to embedded languages
fn do_parse_embedded<'a>(
    corpus: &str,
    language: &Language,
    query: &str,
) -> Result<HashMap<Language, tree_sitter::Tree>, ParseError> {
    let mut out: HashMap<Language, tree_sitter::Tree> = HashMap::new();

    let source = corpus.as_bytes();

    let lang = get_language(language);

    let mut parser = Parser::new();

    if cfg!(debug_assertions) {
        parser.set_logger(Some(Box::new(|log_type, msg| {
            println!("[{:?}]\t{}", log_type, msg);
        })));
    }

    parser
        .set_language(lang)
        .map_err(|_| ParseError::LanguageError)?;

    let tree = parser.parse(&corpus, None).ok_or(ParseError::ParseError)?;

    out.insert(language.to_owned(), tree.to_owned());

    let query = Query::new(lang, query).map_err(|err| ParseError::QueryError {
        message: err.message,
    })?;

    #[derive(Debug)]
    struct Injection {
        ranges: Vec<tree_sitter::Range>,
        language: Language,
    }

    let mut injected_languages: HashMap<usize, Injection> = HashMap::new();

    for pattern_idx in 0..query.pattern_count() {
        for setting in query.property_settings(pattern_idx) {
            match setting.key.as_ref() {
                "injection.language" => {
                    let injection_language = setting.value.as_ref().map(|v| v.as_ref());

                    let language = match injection_language {
                        Some(language_string) => get_language_from_string(language_string),
                        _ => panic!("Unknown injection language"),
                    };

                    injected_languages.insert(
                        pattern_idx,
                        Injection {
                            ranges: Vec::new(),
                            language,
                        },
                    );
                }
                _ => {}
            }
        }
    }

    let mut cursor = QueryCursor::new();

    for query_match in cursor.matches(&query, tree.root_node(), source) {
        match injected_languages.get_mut(&query_match.pattern_index) {
            Some(injection) => {
                for capture in query_match.captures {
                    injection.ranges.push(capture.node.range());
                }
            }
            None => {}
        }
    }

    for (_, injection) in injected_languages {
        let lang = get_language(&injection.language);
        parser.set_language(lang).unwrap();
        parser.set_included_ranges(&injection.ranges).unwrap();
        let tree = parser.parse(&source, None).unwrap();

        out.insert(injection.language.to_owned(), tree.to_owned());
    }

    return Ok(out);
}

#[rustler::nif]
fn parse_embedded(
    corpus: &str,
    language: Language,
    query: &str,
) -> Result<HashMap<Language, TSNode>, ParseError> {
    let source = &corpus.as_bytes();

    return do_parse_embedded(&corpus, &language, &query).map(|map| {
        map.iter()
            .map(|(language, tree)| (language.to_owned(), TSNode::from(tree.root_node(), &source)))
            .collect::<HashMap<_, _>>()
    });
}

// r#"
// ((content) @injection.content
//  (#set! injection.language "html"))
//
// ((code) @injection.content
//  (#set! injection.language "liquid"))
// "#,

#[cfg(test)]
mod tests {

    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_parse_javascript() {
        let corpus = String::from("let x = 1;");
        let result = do_parse(corpus, Language::Javascript);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_parse_liquid() {
        let corpus = String::from("{{ x | y | z}}");

        let result = do_parse(corpus, Language::Liquid);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_parse_css() {
        let corpus = String::from("body .a .b {}");
        let result = do_parse(corpus, Language::Css);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_parse_html() {
        let corpus = String::from("<html></html>");
        let result = do_parse(corpus, Language::Css);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_embedded_html_liquid() {
        let corpus = String::from("<html>{% if a %}<span>{{ 1 }}</span>{% endif %}</html>");

        let result = do_parse_embedded(
            &corpus,
            &Language::LiquidTemplate,
            r#"
            ((content) @injection.content
             (#set! injection.language "html"))

            ((code) @injection.content
             (#set! injection.language "liquid"))
            "#,
        );

        assert_matches!(result, Ok(_));

        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_injected_liquid_html_with_comments() {
        let corpus = String::from("a {{ x }} b {{ # comment }} c {{ y }} d");

        let result = do_parse_embedded(
            &corpus,
            &Language::LiquidTemplate,
            r#"
            ((content) @injection.content
             (#set! injection.language "html"))

            ((code) @injection.content
             (#set! injection.language "liquid"))
            "#,
        );

        let trees = result.unwrap();

        assert_eq!(
            trees[&Language::Liquid].root_node().to_sexp(),
            "(program (identifier) (comment) (identifier))"
        );
    }

    #[test]
    fn test_to_tokens() {
        let corpus = String::from("<html></html>");
        let result = do_to_tokens(corpus, Language::Html);
        assert_eq!(
            result.unwrap(),
            [
                Token {
                    kind: String::from("<"),
                    value: String::from("<"),
                    node_type: NodeType::Anonymous
                },
                Token {
                    kind: String::from("tag_name"),
                    value: String::from("html"),
                    node_type: NodeType::Named
                },
                Token {
                    kind: String::from(">"),
                    value: String::from(">"),
                    node_type: NodeType::Anonymous
                },
                Token {
                    kind: String::from("</"),
                    value: String::from("</"),
                    node_type: NodeType::Anonymous
                },
                Token {
                    kind: String::from("tag_name"),
                    value: String::from("html"),
                    node_type: NodeType::Named
                },
                Token {
                    kind: String::from(">"),
                    value: String::from(">"),
                    node_type: NodeType::Anonymous
                }
            ]
        );
    }

    fn pretty_print_sexp(s: &str) -> () {
        let mut depth = 0;
        for c in s.chars() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
            }
            if c == '(' {
                print!("\n{}", "  ".repeat(depth));
            }
            print!("{}", c);
        }
        println!();
    }
}

rustler::init!(
    "Elixir.TreeSitter",
    [parse, to_sexp, to_tokens, parse_embedded]
);
