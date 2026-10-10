use crate::{cli::Cli, input::InputError};
use clap::ValueEnum;
use ratatui::style::{Color, Style};
use serde::Deserialize;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Clone, Copy, Debug, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Colour {
    Auto,
    Always,
    Never,
}
#[derive(Clone, Copy, Debug, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Auto,
    Dark,
    Light,
}

#[derive(Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserConfig {
    colour: Option<Colour>,
    theme: Option<Theme>,
    line_numbers: Option<bool>,
    #[serde(default)]
    colours: Overrides,
}
#[derive(Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Overrides {
    heading: Option<String>,
    accent: Option<String>,
    secondary: Option<String>,
    value: Option<String>,
    success: Option<String>,
    error: Option<String>,
    process: Option<String>,
}
#[derive(Deserialize)]
struct Resource {
    colours: Colours,
}
#[derive(Clone, Debug, Deserialize)]
struct Colours {
    heading: String,
    accent: String,
    secondary: String,
    value: String,
    success: String,
    error: String,
    process: String,
}
#[derive(Clone, Copy, Debug)]
pub enum Role {
    Heading,
    Accent,
    Secondary,
    Value,
    Success,
    Error,
    Process,
}
#[derive(Clone, Debug)]
pub struct PresentationConfig {
    pub coloured: bool,
    pub line_numbers: bool,
    pub theme: Theme,
    palette: [Color; 7],
}
impl PresentationConfig {
    pub fn style(&self, role: Role) -> Style {
        if !self.coloured {
            return Style::default();
        }
        Style::default().fg(self.palette[role as usize])
    }
    /// Flags > file > environment > terminal detection; redirect always plain.
    pub fn load(
        cli: &Cli,
        stdout_tty: bool,
        cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self, InputError> {
        use std::sync::{atomic::Ordering, mpsc};
        let (tx, rx) = mpsc::sync_channel(1);
        let cli = cli.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Self::load_file(&cli, stdout_tty));
        });
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err(InputError::new(130, "lectura de configuración cancelada"));
            }
            match rx.recv_timeout(std::time::Duration::from_millis(25)) {
                Ok(value) => return value,
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(bad("falló el lector de configuración"));
                }
            }
        }
    }
    fn load_file(cli: &Cli, stdout_tty: bool) -> Result<Self, InputError> {
        let path = cli.config.clone().or_else(default_path);
        let file = if let Some(path) = path {
            match std::fs::metadata(&path) {
                Ok(metadata) if !metadata.is_file() => {
                    return Err(bad("configuración debe ser un archivo regular"));
                }
                Ok(_) => (),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound && cli.config.is_none() => {
                    return Self::resolve(
                        cli,
                        UserConfig::default(),
                        stdout_tty,
                        std::env::var_os("NO_COLOR").is_some(),
                        std::env::var_os("TERM").is_some_and(|x| x == "dumb"),
                    );
                }
                Err(e) => {
                    return Err(bad(format!(
                        "no se puede inspeccionar configuración {}: {e}",
                        path.display()
                    )));
                }
            }
            match File::open(&path) {
                Ok(file) => {
                    let mut bytes = Vec::new();
                    file.take(65537).read_to_end(&mut bytes).map_err(|e| {
                        bad(format!(
                            "no se puede leer configuración {}: {e}",
                            path.display()
                        ))
                    })?;
                    if bytes.len() > 65536 {
                        return Err(bad("configuración excede64KiB"));
                    }
                    let text = std::str::from_utf8(&bytes)
                        .map_err(|e| bad(format!("configuración no UTF-8: {e}")))?;
                    toml::from_str::<UserConfig>(text).map_err(|e| {
                        bad(format!("configuración inválida {}: {e}", path.display()))
                    })?
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound && cli.config.is_none() => {
                    UserConfig::default()
                }
                Err(e) => {
                    return Err(bad(format!(
                        "no se puede abrir configuración {}: {e}",
                        path.display()
                    )));
                }
            }
        } else {
            UserConfig::default()
        };
        let no_colour = std::env::var_os("NO_COLOR").is_some();
        let dumb = std::env::var_os("TERM").is_some_and(|x| x == "dumb");
        Self::resolve(cli, file, stdout_tty, no_colour, dumb)
    }
    fn resolve(
        cli: &Cli,
        file: UserConfig,
        tty: bool,
        no_colour: bool,
        dumb: bool,
    ) -> Result<Self, InputError> {
        let theme = cli.theme.or(file.theme).unwrap_or(Theme::Auto);
        let colour = cli.colour.or(file.colour).unwrap_or(Colour::Auto);
        let coloured = tty
            && match colour {
                Colour::Always => true,
                Colour::Never => false,
                Colour::Auto => !no_colour && !dumb,
            };
        let line_numbers = if cli.line_numbers {
            true
        } else if cli.no_line_numbers {
            false
        } else {
            file.line_numbers.unwrap_or(false)
        };
        let resource = if theme == Theme::Light {
            include_str!("../resources/light_theme.toml")
        } else {
            include_str!("../vendor/r3cli/default_theme.toml")
        };
        let mut colours = toml::from_str::<Resource>(resource)
            .map_err(|e| bad(format!("paleta incluida inválida: {e}")))?
            .colours;
        let overrides = file.colours;
        let override_value = overrides.value.is_some();
        let role_overrides = [
            overrides.heading.is_some(),
            overrides.accent.is_some(),
            overrides.secondary.is_some(),
            overrides.value.is_some(),
            overrides.success.is_some(),
            overrides.error.is_some(),
            overrides.process.is_some(),
        ];
        let inherited = if theme == Theme::Auto {
            cli.nu_palette_value.unwrap_or_default()
        } else {
            crate::nu_palette::NuPalette::default()
        };
        if let Some(x) = overrides.heading {
            colours.heading = x;
        }
        if let Some(x) = overrides.accent {
            colours.accent = x;
        }
        if let Some(x) = overrides.secondary {
            colours.secondary = x;
        }
        if let Some(x) = overrides.value {
            colours.value = x;
        }
        if let Some(x) = overrides.success {
            colours.success = x;
        }
        if let Some(x) = overrides.error {
            colours.error = x;
        }
        if let Some(x) = overrides.process {
            colours.process = x;
        }
        let values = [
            colours.heading,
            colours.accent,
            colours.secondary,
            colours.value,
            colours.success,
            colours.error,
            colours.process,
        ];
        let mut palette = [Color::Reset; 7];
        for (index, value) in values.iter().enumerate() {
            palette[index] = parse_colour(value)?;
        }
        for (index, colour) in inherited.roles.iter().enumerate() {
            if !role_overrides[index]
                && let Some(colour) = colour
            {
                palette[index] = *colour;
            }
        }
        // Auto inherits terminal foreground rather than assuming dark background.
        if theme == Theme::Auto
            && !override_value
            && inherited.roles[Role::Value as usize].is_none()
        {
            palette[Role::Value as usize] = Color::Reset;
        }
        Ok(Self {
            coloured,
            line_numbers,
            theme,
            palette,
        })
    }
}
fn bad(message: impl Into<String>) -> InputError {
    InputError::new(2, message)
}
fn parse_colour(value: &str) -> Result<Color, InputError> {
    if value == "default" {
        return Ok(Color::Reset);
    }
    let Some(hex) = value.strip_prefix('#') else {
        return Err(bad("color debe ser #RRGGBB o default"));
    };
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("color debe ser #RRGGBB o default"));
    }
    Ok(Color::Rgb(
        u8::from_str_radix(&hex[0..2], 16).unwrap(),
        u8::from_str_radix(&hex[2..4], 16).unwrap(),
        u8::from_str_radix(&hex[4..6], 16).unwrap(),
    ))
}
fn default_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|x| PathBuf::from(x).join("R3Viewer/config.toml"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|x| PathBuf::from(x).join("Library/Application Support/R3Viewer/config.toml"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|x| PathBuf::from(x).is_absolute())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|x| PathBuf::from(x).join(".config")))
            .map(|x| x.join("r3viewer/config.toml"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn flags_file_environment_and_redirect_precedence() {
        let cli = Cli::parse_from([
            "viewer",
            "--colour",
            "always",
            "--theme",
            "light",
            "--no-line-numbers",
        ]);
        let file = toml::from_str::<UserConfig>("colour='never'\ntheme='dark'\nline_numbers=true")
            .unwrap();
        let config = PresentationConfig::resolve(&cli, file, true, true, true).unwrap();
        assert!(config.coloured);
        assert!(!config.line_numbers);
        assert_eq!(config.theme, Theme::Light);
        let config =
            PresentationConfig::resolve(&cli, UserConfig::default(), false, false, false).unwrap();
        assert!(!config.coloured);
        let auto = Cli::parse_from(["viewer"]);
        assert!(
            !PresentationConfig::resolve(&auto, UserConfig::default(), true, true, false)
                .unwrap()
                .coloured
        );
        assert_eq!(
            PresentationConfig::resolve(&auto, UserConfig::default(), true, false, false)
                .unwrap()
                .palette[3],
            Color::Reset
        );
    }
    #[test]
    fn invalid_keys_and_colours_do_not_silently_fall_back() {
        assert!(toml::from_str::<UserConfig>("colur='always'").is_err());
        let cli = Cli::parse_from(["viewer"]);
        let file = toml::from_str::<UserConfig>("[colours]\nheading='red'").unwrap();
        assert!(PresentationConfig::resolve(&cli, file, false, false, false).is_err());
        assert!(parse_colour("#🦊ab").is_err());
    }
    #[test]
    fn nu_foreground_precedence_and_auto_value_presence() {
        let raw = r#"{"v":1,"roles":{"heading":{"ansi":2},"value":{"rgb":[18,52,86]},"success":{"default":true}}}"#;
        let mut cli = Cli::parse_from(["viewer", "--nu-wire", "--nu-palette", raw]);
        cli.prepare_metadata().unwrap();
        assert!(cli.nu_palette.is_none());
        let c =
            PresentationConfig::resolve(&cli, UserConfig::default(), true, false, false).unwrap();
        assert_eq!(c.palette[0], Color::Indexed(2));
        assert_eq!(c.palette[3], Color::Rgb(18, 52, 86));
        assert_eq!(c.palette[4], Color::Reset);
        assert_eq!(c.palette[5], Color::Rgb(245, 90, 90));
        let file =
            toml::from_str::<UserConfig>("[colours]\nvalue='#aabbcc'\nheading='default'").unwrap();
        let c = PresentationConfig::resolve(&cli, file, true, false, false).unwrap();
        assert_eq!(c.palette[0], Color::Reset);
        assert_eq!(c.palette[3], Color::Rgb(170, 187, 204));
        cli.theme = Some(Theme::Light);
        let c =
            PresentationConfig::resolve(&cli, UserConfig::default(), true, false, false).unwrap();
        assert_eq!(c.palette[0], Color::Rgb(18, 104, 117));
        assert_eq!(c.palette[3], Color::Rgb(32, 36, 43));
        cli.theme = Some(Theme::Auto);
        cli.colour = Some(Colour::Never);
        let c =
            PresentationConfig::resolve(&cli, UserConfig::default(), true, false, false).unwrap();
        assert_eq!(c.style(Role::Heading), Style::default());
        cli.colour = Some(Colour::Always);
        let c =
            PresentationConfig::resolve(&cli, UserConfig::default(), false, false, false).unwrap();
        assert_eq!(c.style(Role::Heading), Style::default());
    }
}
