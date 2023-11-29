// use std::path::Path;

fn main() {
//     let languages = vec!["liquid", "liquid-template"];
//
//     for language in languages {
//         let package = format!("tree-sitter-{}", language);
//         let source_directory = format!("{}/src", package);
//         let parser_file = format!("{}/parser.c", source_directory);
//         let scanner_file = format!("{}/scanner.c", source_directory);
//         let grammar_file = format!("{}/grammar.js", package);

//         let mut build = cc::Build::new();

//         // Check if scanner.c exists
//         if Path::new(&scanner_file).exists() {
//             println!("cargo:rerun-if-changed={}", scanner_file);
//             build.file(scanner_file);
//         }

//         println!("cargo:rerun-if-changed={}", parser_file);
//         println!("cargo:rerun-if-changed={}", grammar_file);

//         build
//             .file(parser_file)
//             .include(source_directory)
//             .compile(&package);
//     }
}
