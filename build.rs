fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let resources = [
        (
            "resources/syntax/toml/TOML.sublime-syntax",
            include_str!("resources/syntax/toml/TOML.sublime-syntax"),
        ),
        (
            "resources/syntax/nushell/Nushell.sublime-syntax",
            include_str!("resources/syntax/nushell/Nushell.sublime-syntax"),
        ),
        (
            "resources/syntax/powershell/PowerShell.sublime-syntax",
            include_str!("resources/syntax/powershell/PowerShell.sublime-syntax"),
        ),
    ];
    let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
    for (path, source) in resources {
        println!("cargo:rerun-if-changed={path}");
        let definition = syntect::parsing::SyntaxDefinition::load_from_str(source, true, None)
            .unwrap_or_else(|error| panic!("fixed syntax {path} must compile: {error}"));
        builder.add(definition);
    }
    let syntaxes = builder.build();
    let unlinked = syntaxes.find_unlinked_contexts();
    assert!(
        unlinked.is_empty(),
        "fixed syntax references must link: {unlinked:?}"
    );
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    syntect::dumps::dump_to_file(&syntaxes, output.join("bundled.syntax.pack"))
        .expect("write embedded fixed syntax dump");
    // Preserve SyntaxSet order: extension resolution uses its reverse precedence.
    let mut catalog = String::from("pub const LANGUAGES: &[(&str, &[&str])] = &[\n");
    for syntax in syntaxes.syntaxes() {
        catalog.push_str(&format!("    ({:?}, &[", syntax.name));
        for extension in &syntax.file_extensions {
            catalog.push_str(&format!("{extension:?},"));
        }
        catalog.push_str("]),\n");
    }
    catalog.push_str("];\n");
    std::fs::write(output.join("language_catalog.rs"), catalog)
        .expect("write catalog from the same fixed syntax set");
}
