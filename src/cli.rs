use clap::{Parser, ValueEnum};

/// Visor de archivos y contenido. S01: lectura textual UTF-8.
#[derive(Clone, Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Archivo local; con --value se interpreta como contenido literal.
    pub source: Option<String>,
    /// Transporte privado del companion Nushell.
    #[arg(long, hide = true)]
    pub nu_wire: bool,
    /// Imprimir en el scrollback y terminar.
    #[arg(short = 'i', long)]
    pub inline: bool,
    /// Forzar ruta local (incompatible con --value).
    #[arg(short = 'p', long, conflicts_with = "value")]
    pub path: bool,
    /// Forzar contenido literal, también desde stdin.
    #[arg(short = 'v', long)]
    pub value: bool,
    /// Formato explícito. Este incremento incluye solamente text.
    #[arg(long, value_enum, default_value = "text")]
    pub format: Format,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    Text,
}
