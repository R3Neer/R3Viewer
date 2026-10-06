//! Allocation-free scalar projection shared by Source and Document.
pub(crate) fn scalar(ch: char, next: Option<char>, out: &mut [u8; 8]) -> &str {
    let ch = match ch {
        '\n' | '\t' => ch,
        '\r' if next == Some('\n') => ch,
        '\u{7f}' => '␡',
        '\u{0}'..='\u{1f}' => char::from_u32(0x2400 + ch as u32).unwrap(),
        '\u{80}'..='\u{9f}' => {
            const HEX: &[u8; 16] = b"0123456789abcdef";
            *out = *b"\\u{0000}";
            let n = ch as usize;
            out[5] = HEX[(n >> 4) & 15];
            out[6] = HEX[n & 15];
            return std::str::from_utf8(out).unwrap();
        }
        _ => ch,
    };
    ch.encode_utf8(out)
}
pub(crate) fn len(ch: char, next: Option<char>) -> usize {
    scalar(ch, next, &mut [0; 8]).len()
}
pub(crate) fn append(out: &mut String, ch: char, next: Option<char>) {
    out.push_str(scalar(ch, next, &mut [0; 8]));
}
