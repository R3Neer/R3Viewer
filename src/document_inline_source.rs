//! Complete safe Source fallback for document inline; no indexes, output writes or ANSI.
use std::sync::atomic::{AtomicBool, Ordering};
const MAX_OUTPUT: usize = 256 * 1024 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Cancelled,
    OutputLimit,
    Allocation,
}
fn check(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
/// Count first, then allocate one candidate and copy scalars. The caller writes only Ok.
pub(crate) fn prepare(source: &str, cancel: &AtomicBool, limit: usize) -> Result<String, Error> {
    check(cancel)?;
    let limit = limit.min(MAX_OUTPUT);
    let mut needed = 0usize;
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        check(cancel)?;
        needed = needed
            .checked_add(crate::text_projection::len(ch, chars.peek().copied()))
            .filter(|&n| n <= limit)
            .ok_or(Error::OutputLimit)?;
    }
    check(cancel)?;
    let mut candidate = String::new();
    candidate
        .try_reserve_exact(needed)
        .map_err(|_| Error::Allocation)?;
    // Reserve_exact can return a larger capacity; reject before copying, never call it exact heap use.
    if candidate.capacity() > limit {
        return Err(Error::OutputLimit);
    }
    let mut chars = source.chars().peekable();
    let mut scratch = [0u8; 8];
    while let Some(ch) = chars.next() {
        check(cancel)?;
        // Each safe scalar is at most eight UTF-8 bytes, below the 4096-byte copy boundary.
        candidate.push_str(crate::text_projection::scalar(
            ch,
            chars.peek().copied(),
            &mut scratch,
        ));
    }
    check(cancel)?;
    if candidate.len() != needed {
        return Err(Error::Allocation);
    }
    Ok(candidate)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_scalars_preserve_crlf_tabs_unicode_and_literal_placeholders() {
        let source = "\0\u{1b}\r\n\u{7f}\u{80}\t🦀␛\\u{0080}";
        let expected = "␀␛\r\n␡\\u{0080}\t🦀␛\\u{0080}";
        assert_eq!(
            prepare(source, &AtomicBool::new(false), expected.len()).unwrap(),
            expected
        );
        assert_eq!(source, "\0\u{1b}\r\n\u{7f}\u{80}\t🦀␛\\u{0080}");
        assert_eq!(
            prepare(source, &AtomicBool::new(false), expected.len() - 1),
            Err(Error::OutputLimit)
        );
    }
    #[test]
    fn empty_cancel_and_projection_expansion_have_transactional_results() {
        assert_eq!(prepare("", &AtomicBool::new(false), 0).unwrap(), "");
        assert_eq!(
            prepare("abc", &AtomicBool::new(true), 3),
            Err(Error::Cancelled)
        );
        assert_eq!(
            prepare("\u{80}", &AtomicBool::new(false), 7),
            Err(Error::OutputLimit)
        );
        assert_eq!(
            prepare("\u{80}", &AtomicBool::new(false), 8).unwrap(),
            "\\u{0080}"
        );
        assert_eq!(prepare("\r", &AtomicBool::new(false), 3).unwrap(), "␍");
    }
    #[test]
    fn no_tui_line_limit_and_no_manufactured_terminal_newline() {
        let source = "\n".repeat(1_000_000);
        assert_eq!(
            prepare(&source, &AtomicBool::new(false), source.len()).unwrap(),
            source
        );
        assert_eq!(prepare("last", &AtomicBool::new(false), 4).unwrap(), "last");
        assert_eq!(
            prepare("last\n", &AtomicBool::new(false), 5).unwrap(),
            "last\n"
        );
    }
}
