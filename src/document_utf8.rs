//! Bounded UTF8 proof before a borrowed string is constructed.
use crate::{
    document_model::{Error, ErrorKind, Result},
    document_obsidian::Control,
};
pub(crate) fn checked<'a>(bytes: &'a [u8], ctl: &Control<'_>) -> Result<&'a str> {
    let mut start = 0;
    while start < bytes.len() {
        ctl.poll()?;
        let mut end = (start + 4096).min(bytes.len());
        if end < bytes.len() {
            while end > start && bytes[end] & 0xc0 == 0x80 {
                end -= 1;
            }
        }
        if end == start || std::str::from_utf8(&bytes[start..end]).is_err() {
            return Err(Error::new(ErrorKind::InvalidContext));
        }
        start = end;
    }
    ctl.poll()?;
    // Every byte belongs to an individually proven UTF8 chunk, never split inside a scalar.
    Ok(unsafe { std::str::from_utf8_unchecked(bytes) })
}
