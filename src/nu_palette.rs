//! Closed presentation metadata; never part of the document wire or model.
use jiter::Jiter;
use ratatui::style::Color;

pub const MAX_METADATA_BYTES: usize = 1024;
const ROLES: [&[u8]; 7] = [
    b"heading",
    b"accent",
    b"secondary",
    b"value",
    b"success",
    b"error",
    b"process",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NuPalette {
    pub(crate) roles: [Option<Color>; 7],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidPalette;

impl NuPalette {
    pub fn parse(raw: &str) -> Result<Self, InvalidPalette> {
        // Producer schema contains no escaped strings. Reject before Jiter's string tape.
        if raw.len() > MAX_METADATA_BYTES || !raw.is_ascii() || raw.as_bytes().contains(&b'\\') {
            return Err(InvalidPalette);
        }
        let mut parser = Jiter::new(raw.as_bytes());
        let mut palette = Self::default();
        let mut seen = 0u8;
        let mut key = parser.next_object_bytes().map_err(|_| InvalidPalette)?;
        while let Some(k) = key {
            let bit = match k {
                b"v" => 1,
                b"roles" => 2,
                _ => return Err(InvalidPalette),
            };
            if seen & bit != 0 {
                return Err(InvalidPalette);
            }
            seen |= bit;
            if bit == 1 {
                if parser.next_number_bytes().map_err(|_| InvalidPalette)? != b"1" {
                    return Err(InvalidPalette);
                }
            } else {
                let mut role_seen = 0u8;
                let mut role_key = parser.next_object_bytes().map_err(|_| InvalidPalette)?;
                while let Some(k) = role_key {
                    let index = ROLES
                        .iter()
                        .position(|name| *name == k)
                        .ok_or(InvalidPalette)?;
                    let mask = 1u8 << index;
                    if role_seen & mask != 0 {
                        return Err(InvalidPalette);
                    }
                    role_seen |= mask;
                    palette.roles[index] = Some(parse_color(&mut parser)?);
                    role_key = parser.next_key_bytes().map_err(|_| InvalidPalette)?;
                }
            }
            key = parser.next_key_bytes().map_err(|_| InvalidPalette)?;
        }
        if seen != 3 {
            return Err(InvalidPalette);
        }
        parser.finish().map_err(|_| InvalidPalette)?;
        Ok(palette)
    }
}
fn byte(parser: &mut Jiter<'_>, max: u8) -> Result<u8, InvalidPalette> {
    let raw = parser.next_number_bytes().map_err(|_| InvalidPalette)?;
    if raw.is_empty() || raw.len() > 3 || !raw.iter().all(u8::is_ascii_digit) {
        return Err(InvalidPalette);
    }
    let mut value = 0u16;
    for b in raw {
        value = value * 10 + u16::from(*b - b'0');
    }
    if value > u16::from(max) {
        return Err(InvalidPalette);
    }
    Ok(value as u8)
}
fn parse_color(parser: &mut Jiter<'_>) -> Result<Color, InvalidPalette> {
    let key = parser
        .next_object_bytes()
        .map_err(|_| InvalidPalette)?
        .ok_or(InvalidPalette)?;
    let color = match key {
        b"ansi" => Color::Indexed(byte(parser, 15)?),
        b"default" => {
            if !parser.next_bool().map_err(|_| InvalidPalette)? {
                return Err(InvalidPalette);
            }
            Color::Reset
        }
        b"rgb" => {
            if parser.next_array().map_err(|_| InvalidPalette)?.is_none() {
                return Err(InvalidPalette);
            }
            let red = byte(parser, 255)?;
            if parser.array_step().map_err(|_| InvalidPalette)?.is_none() {
                return Err(InvalidPalette);
            }
            let green = byte(parser, 255)?;
            if parser.array_step().map_err(|_| InvalidPalette)?.is_none() {
                return Err(InvalidPalette);
            }
            let blue = byte(parser, 255)?;
            if parser.array_step().map_err(|_| InvalidPalette)?.is_some() {
                return Err(InvalidPalette);
            }
            Color::Rgb(red, green, blue)
        }
        _ => return Err(InvalidPalette),
    };
    if parser
        .next_key_bytes()
        .map_err(|_| InvalidPalette)?
        .is_some()
    {
        return Err(InvalidPalette);
    }
    Ok(color)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_metadata() {
        let p = NuPalette::parse(r#"{"v":1,"roles":{"heading":{"ansi":7},"value":{"rgb":[18,52,86]},"secondary":{"default":true}}}"#).unwrap();
        assert_eq!(p.roles[0], Some(Color::Indexed(7)));
        assert_eq!(p.roles[3], Some(Color::Rgb(18, 52, 86)));
        assert_eq!(p.roles[2], Some(Color::Reset));
        assert_eq!(p.roles[5], None);
        assert_eq!(
            NuPalette::parse(r#"{"roles":{},"v":1}"#).unwrap(),
            NuPalette::default()
        );
        let empty = r#"{"v":1,"roles":{}}"#;
        let limit = format!("{empty}{}", " ".repeat(1024 - empty.len()));
        assert!(NuPalette::parse(&limit).is_ok());
        assert!(NuPalette::parse(&(limit + " ")).is_err());
        for bad in [
            r#"{"v":1,"v":1,"roles":{}}"#,
            r#"{"v":2,"roles":{}}"#,
            r#"{"v":1.0,"roles":{}}"#,
            r#"{"v":1,"roles":{"heading":{"ansi":16}}}"#,
            r#"{"v":1,"roles":{"heading":{"ansi":-1}}}"#,
            r#"{"v":1,"roles":{"heading":{"ansi":1.0}}}"#,
            r#"{"v":1,"roles":{"heading":{"default":false}}}"#,
            r#"{"v":1,"roles":{"value":{"rgb":[1,2,256]}}}"#,
            r#"{"v":1,"roles":{"value":{"rgb":[1,2]}}}"#,
            r#"{"v":1,"roles":{"value":{"rgb":[1,2,3,4]}}}"#,
            r#"{"v":1,"roles":{"value":{"ansi":1,"ansi":1}}}"#,
            r#"{"v":1,"roles":{"value":{"ansi":1},"value":{"ansi":1}}}"#,
            r#"{"v":1,"roles":{"unknown":{"ansi":1}}}"#,
            r#"{"v":1,"roles":{},"extra":0}"#,
            r#"{"v":1,"roles":{}}x"#,
            r#"{"\u0076":1,"roles":{}}"#,
        ] {
            assert!(NuPalette::parse(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn compiled_metadata_layout_inventory() {
        println!(
            "C01_LAYOUT NuPalette={} OptionNuPalette={} Color={} OptionColor={} Jiter={} Cli={} roles7={} MAX_METADATA={}",
            std::mem::size_of::<NuPalette>(),
            std::mem::size_of::<Option<NuPalette>>(),
            std::mem::size_of::<Color>(),
            std::mem::size_of::<Option<Color>>(),
            std::mem::size_of::<Jiter<'_>>(),
            std::mem::size_of::<crate::cli::Cli>(),
            std::mem::size_of::<[Option<Color>; 7]>(),
            MAX_METADATA_BYTES
        );
    }
}
