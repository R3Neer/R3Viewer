use clap::{Parser, ValueEnum};

/// Visor de archivos y datos, de solo lectura.
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
    /// Archivo TOML de configuración; no modifica perfiles.
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,
    /// Color de presentación (la redirección siempre es texto plano).
    #[arg(long, value_enum)]
    pub colour: Option<crate::config::Colour>,
    /// Paleta; auto conserva el foreground del terminal.
    #[arg(long, value_enum)]
    pub theme: Option<crate::config::Theme>,
    /// Mostrar números de línea.
    #[arg(short = 'n', long, conflicts_with = "no_line_numbers")]
    pub line_numbers: bool,
    /// Ocultar números de línea configurados.
    #[arg(long)]
    pub no_line_numbers: bool,
    /// Formato explícito; auto selecciona una presentación compatible.
    #[arg(long, value_enum, default_value = "auto")]
    pub format: Format,
    /// Gramática integrada por nombre o extensión; nunca una ruta.
    #[arg(long)]
    pub language: Option<String>,
    /// Mostrar inicialmente la estructura interpretada, cuando el formato la permite.
    #[arg(long)]
    pub structure: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Format {
    Auto,
    Text,
    Code,
    Toml,
    Json,
    Yaml,
    Xml,
}

impl Cli {
    /// Validate option semantics before configuration or document I/O.
    pub fn validate(&self) -> Result<(), crate::input::InputError> {
        if self.structure && self.format == Format::Text {
            return Err(crate::input::InputError::new(
                2,
                "elige --format text o --structure; no pueden coexistir",
            ));
        }
        if let Some(language) = &self.language {
            if self.format == Format::Text {
                return Err(crate::input::InputError::new(
                    2,
                    "elige --format text o --language; no pueden coexistir",
                ));
            }
            if language.trim().is_empty() || language.len() > 256 {
                return Err(crate::input::InputError::new(
                    2,
                    "--language requiere un nombre no vacío de hasta 256 bytes",
                ));
            }
        }
        Ok(())
    }
}
