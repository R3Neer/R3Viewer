use std::sync::atomic::{AtomicBool, Ordering};

fn sanitize(source: &str, mut cancelled: impl FnMut() -> bool) -> Option<String> {
    if cancelled() {
        return None;
    }
    let mut result = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut index = 0usize;
    while let Some(ch) = chars.next() {
        if index.is_multiple_of(4096) && cancelled() {
            return None;
        }
        index += 1;
        match ch {
            '\n' | '\t' => result.push(ch),
            '\r' if chars.peek() == Some(&'\n') => result.push(ch),
            '\u{7f}' => result.push('␡'),
            '\u{0}'..='\u{1f}' => result.push(char::from_u32(0x2400 + ch as u32).unwrap()),
            '\u{80}'..='\u{9f}' => result.push_str(&format!("\\u{{{:04x}}}", ch as u32)),
            _ => result.push(ch),
        }
    }
    if cancelled() { None } else { Some(result) }
}

/// Conserva texto y saltos; convierte controles ejecutables en texto visible.
pub fn safe_text(source: &str) -> String {
    sanitize(source, || false).expect("el sanitizado sin cancelación siempre termina")
}

pub fn safe_text_cancelable(source: &str, cancelled: &AtomicBool) -> Option<String> {
    sanitize(source, || cancelled.load(Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_unicode_and_layout_but_cannot_emit_terminal_commands() {
        assert_eq!(safe_text("á界\t\r\n\n"), "á界\t\r\n\n");
        let safe = safe_text("\x1b]0;title\x07\rhello\u{009b}31m\x7f");
        assert!(
            !safe
                .chars()
                .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
        );
        assert!(safe.starts_with("␛]0;title␇␍hello"));
    }
    #[test]
    fn cancellation_is_observed_during_processing_not_only_before_it() {
        let mut calls = 0;
        let source = "\0".repeat(32768);
        let result = sanitize(&source, || {
            calls += 1;
            calls >= 3
        });
        assert!(result.is_none());
        assert_eq!(calls, 3);
        assert!(safe_text_cancelable("", &AtomicBool::new(true)).is_none());
    }
}
