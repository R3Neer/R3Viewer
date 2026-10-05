fn main() {
    let resource = "resources/syntax/toml/TOML.sublime-syntax";
    println!("cargo:rerun-if-changed={resource}");
    let definition = syntect::parsing::SyntaxDefinition::load_from_str(
        include_str!("resources/syntax/toml/TOML.sublime-syntax"),
        true,
        None,
    )
    .expect("fixed TOML syntax resource must compile");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
    builder.add(definition);
    let syntaxes = builder.build();
    syntect::dumps::dump_to_file(&syntaxes, output.join("toml.syntax.pack"))
        .expect("write embedded TOML syntax dump");
}
