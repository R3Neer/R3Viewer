use std::time::Instant;
pub type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Start,
    End,
    Empty,
    Text,
    Ref,
    CData,
    Comment,
    Pi,
    Decl,
    DocType,
}
#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
    pub payload: Span,
}
pub struct Control<'a> {
    pub deadline: Instant,
    pub cancel: &'a mut dyn FnMut() -> bool,
    pub own: usize,
    pub base: usize,
    pub limit: usize,
}
impl Control<'_> {
    pub fn poll(&mut self) -> Result<()> {
        if (self.cancel)() {
            return Err("Cancelled");
        }
        if Instant::now() >= self.deadline {
            return Err("Timeout");
        }
        Ok(())
    }
    pub fn charge(&mut self, n: usize) -> Result<()> {
        let next = self.own.checked_add(n).ok_or("Budget")?;
        if next.checked_add(self.base).is_none_or(|n| n > self.limit) {
            return Err("Budget");
        }
        self.own = next;
        Ok(())
    }
    pub fn release(&mut self, n: usize) {
        self.own = self.own.checked_sub(n).expect("charged scratch")
    }
}
pub fn is_s(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}
pub fn legal(c: char) -> bool {
    matches!(c, '\t' | '\r' | '\n')
        || matches!(c as u32,0x20..=0xd7ff|0xe000..=0xfffd|0x10000..=0x10ffff)
}
pub fn validate_chars(s: &str, c: &mut Control) -> Result<()> {
    for (i, x) in s.char_indices() {
        if i % 4096 < 4 {
            c.poll()?;
        }
        if !legal(x) {
            return Err("InvalidXml");
        }
    }
    Ok(())
}
pub fn reserve<T>(v: &mut Vec<T>, n: usize, c: &mut Control) -> Result<()> {
    let need = v.len().checked_add(n).ok_or("Budget")?;
    if need > v.capacity() {
        let cap = v
            .capacity()
            .max(8)
            .checked_mul(2)
            .ok_or("Budget")?
            .max(need);
        let old = v.capacity() * std::mem::size_of::<T>();
        let bytes = cap.checked_mul(std::mem::size_of::<T>()).ok_or("Budget")?;
        c.charge(bytes)?;
        v.try_reserve_exact(cap - v.len()).map_err(|_| "Budget")?;
        let actual = v.capacity() * std::mem::size_of::<T>();
        if actual > bytes {
            c.charge(actual - bytes)?
        } else {
            c.release(bytes - actual)
        }
        c.release(old);
    }
    Ok(())
}
pub struct Cursor<'a> {
    pub s: &'a str,
    pub pos: usize,
}
impl<'a> Cursor<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            pos: if s.as_bytes().starts_with(&[239, 187, 191]) {
                3
            } else {
                0
            },
        }
    }
    fn find(&self, at: usize, needle: &[u8], c: &mut Control) -> Result<usize> {
        let b = self.s.as_bytes();
        let mut i = at;
        while i + needle.len() <= b.len() {
            if i & 4095 == 0 {
                c.poll()?;
            }
            if &b[i..i + needle.len()] == needle {
                return Ok(i);
            }
            i += 1;
        }
        Err("InvalidXml")
    }
    pub fn next(&mut self, c: &mut Control) -> Result<Option<Token>> {
        c.poll()?;
        let b = self.s.as_bytes();
        let a = self.pos;
        if a == b.len() {
            return Ok(None);
        }
        let tail = &b[a..];
        let (kind, end, ps, pe) = if tail.starts_with(b"<!--") {
            let e = self.find(a + 4, b"-->", c)?;
            (Kind::Comment, e + 3, a + 4, e)
        } else if tail.starts_with(b"<![CDATA[") {
            let e = self.find(a + 9, b"]]>", c)?;
            (Kind::CData, e + 3, a + 9, e)
        } else if tail.starts_with(b"<?") {
            let e = self.find(a + 2, b"?>", c)?;
            let k = if self.s[a + 2..e].starts_with("xml")
                && (e == a + 5 || b.get(a + 5).is_some_and(|x| is_s(*x)))
            {
                Kind::Decl
            } else {
                Kind::Pi
            };
            (k, e + 2, a + 2, e)
        } else if tail.starts_with(b"<!DOCTYPE") {
            let mut i = a + 9;
            let mut subset = false;
            let mut subset_done = false;
            let mut quote = 0;
            let e = loop {
                if i >= b.len() {
                    return Err("InvalidXml");
                }
                if i & 4095 == 0 {
                    c.poll()?;
                }
                if subset_done && !is_s(b[i]) && b[i] != b'>' {
                    return Err("InvalidXml");
                }
                if quote != 0 {
                    if b[i] == quote {
                        quote = 0;
                    }
                    i += 1;
                    continue;
                }
                if b[i] == b'\'' || b[i] == b'"' {
                    quote = b[i];
                    i += 1;
                    continue;
                }
                if b[i..].starts_with(b"<!--") {
                    i = self.find(i + 4, b"-->", c)? + 3;
                    continue;
                }
                if b[i..].starts_with(b"<?") {
                    i = self.find(i + 2, b"?>", c)? + 2;
                    continue;
                }
                if b[i] == b'[' {
                    if subset {
                        return Err("XmlRepresentation");
                    }
                    subset = true;
                } else if b[i] == b']' {
                    if !subset {
                        return Err("InvalidXml");
                    }
                    subset = false;
                    subset_done = true;
                } else if b[i] == b'>' && !subset {
                    break i;
                }
                i += 1;
            };
            (Kind::DocType, e + 1, a + 9, e)
        } else if tail.starts_with(b"</") {
            let e = self.find(a + 2, b">", c)?;
            (Kind::End, e + 1, a + 2, e)
        } else if tail.starts_with(b"<") {
            if tail.starts_with(b"<!") {
                return Err("InvalidXml");
            }
            let mut i = a + 1;
            let mut quote = 0;
            while i < b.len() {
                if i & 4095 == 0 {
                    c.poll()?;
                }
                if quote != 0 {
                    if b[i] == quote {
                        quote = 0;
                    }
                } else if b[i] == b'\'' || b[i] == b'"' {
                    quote = b[i];
                } else if b[i] == b'>' {
                    break;
                }
                i += 1;
            }
            if i == b.len() {
                return Err("InvalidXml");
            }
            let empty = b.get(i.wrapping_sub(1)) == Some(&b'/');
            (
                if empty { Kind::Empty } else { Kind::Start },
                i + 1,
                a + 1,
                if empty { i - 1 } else { i },
            )
        } else if tail.starts_with(b"&") {
            let e = self.find(a + 1, b";", c)?;
            (Kind::Ref, e + 1, a + 1, e)
        } else {
            let mut i = a;
            while i < b.len() && !matches!(b[i], b'<' | b'&') {
                if i & 4095 == 0 {
                    c.poll()?;
                }
                i += 1;
            }
            (Kind::Text, i, a, i)
        };
        self.pos = end;
        Ok(Some(Token {
            kind,
            span: Span { start: a, end },
            payload: Span { start: ps, end: pe },
        }))
    }
}
