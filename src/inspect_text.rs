//! Complete primitive descriptors; never evaluates values or reopens metadata.
use crate::data::DataRef;
use r3viewer_wire::{ByteStreamKind, CellPathMember, Value};
use std::{
    fmt::Write,
    sync::atomic::{AtomicBool, Ordering},
};
fn check(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("inspección cancelada".into())
    } else {
        Ok(())
    }
}
fn append(out: &mut String, text: &str, cancel: &AtomicBool) -> Result<(), String> {
    let mut offset = 0;
    while offset < text.len() {
        check(cancel)?;
        let mut end = offset.saturating_add(65536).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        out.push_str(&text[offset..end]);
        offset = end;
    }
    Ok(())
}
pub fn copy_text(text: &str, cancel: &AtomicBool) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    append(&mut out, text, cancel)?;
    Ok(out)
}
fn hex(out: &mut String, bytes: &[u8], cancel: &AtomicBool) -> Result<(), String> {
    const HEX: &[u8] = b"0123456789abcdef";
    for chunk in bytes.chunks(65536) {
        check(cancel)?;
        for b in chunk {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 15) as usize] as char);
        }
    }
    Ok(())
}
pub fn primitive_text(node: DataRef<'_>, cancel: &AtomicBool) -> Result<String, String> {
    check(cancel)?;
    let mut out = String::new();
    match node {
        DataRef::Value(Value::String(text)) => append(&mut out, text, cancel)?,
        DataRef::ByteStream {
            kind: ByteStreamKind::String,
            bytes,
        } => append(
            &mut out,
            std::str::from_utf8(bytes).map_err(|_| "string_stream no UTF-8")?,
            cancel,
        )?,
        DataRef::Value(Value::Glob { pattern, no_expand }) => {
            writeln!(out, "glob(no_expand={no_expand}, pattern=").unwrap();
            append(&mut out, pattern, cancel)?;
            out.push(')');
        }
        DataRef::Value(Value::Error { message }) => {
            out.push_str("error(\n");
            append(&mut out, message, cancel)?;
            out.push(')');
        }
        DataRef::Value(Value::CellPath(members)) => {
            out.push_str("cell_path[\n");
            for member in members {
                check(cancel)?;
                match member {
                    CellPathMember::Index { value, optional } => {
                        writeln!(out, "  index={value}, optional={optional}").unwrap()
                    }
                    CellPathMember::String {
                        value,
                        optional,
                        insensitive,
                    } => {
                        write!(
                            out,
                            "  string(optional={optional}, insensitive={insensitive}): "
                        )
                        .unwrap();
                        append(&mut out, value, cancel)?;
                        out.push('\n');
                    }
                }
            }
            out.push(']');
        }
        DataRef::Value(Value::Binary(bytes)) => {
            out.push_str("binary(0x");
            hex(&mut out, bytes, cancel)?;
            out.push(')');
        }
        DataRef::ByteStream { kind, bytes } => {
            out.push_str(if kind == ByteStreamKind::Binary {
                "binary_stream(0x"
            } else {
                "unknown_stream(0x"
            });
            hex(&mut out, bytes, cancel)?;
            out.push(')');
        }
        _ if node.is_container() => {
            return Err("contenedor requiere navegación, no descriptor de hoja".into());
        }
        _ => out.push_str(&node.summary(4096)),
    }
    check(cancel)?;
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_string_controls_and_cellpath_without_execution() {
        let cancel = AtomicBool::new(false);
        let raw = Value::String(format!("{}\x1bend", "🦊".repeat(20000)));
        assert!(
            primitive_text(DataRef::Value(&raw), &cancel)
                .unwrap()
                .ends_with("\x1bend")
        );
        let cell = Value::CellPath(vec![CellPathMember::String {
            value: "hello\x1b".into(),
            optional: true,
            insensitive: true,
        }]);
        let text = primitive_text(DataRef::Value(&cell), &cancel).unwrap();
        assert!(text.contains("hello\x1b"));
        assert!(text.contains("insensitive=true"));
        assert!(primitive_text(DataRef::Value(&raw), &AtomicBool::new(true)).is_err());
    }
}
