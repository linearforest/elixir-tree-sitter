fn main() {
    let language = "liquid";
    let package = format!("tree-sitter-{}", language);
    let source_directory = format!("{}/src", package);
    let source_file = format!("{}/parser.c", source_directory);
    let grammar_file = format!("{}/grammar.js", package);

    println!("cargo:rerun-if-changed={}", source_file);
    println!("cargo:rerun-if-changed={}", grammar_file);

    cc::Build::new()
        .file(source_file)
        .include(source_directory)
        .compile(&package); // <2>
}
