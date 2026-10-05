use super::lexical::*;
pub fn nc_start(x: char) -> bool {
    matches!(x,'A'..='Z'|'a'..='z'|'_')
        || matches!(x as u32,0xc0..=0xd6|0xd8..=0xf6|0xf8..=0x2ff|0x370..=0x37d|0x37f..=0x1fff|0x200c..=0x200d|0x2070..=0x218f|0x2c00..=0x2fef|0x3001..=0xd7ff|0xf900..=0xfdcf|0xfdf0..=0xfffd|0x10000..=0xeffff)
}
pub fn nc_name(s: &str, c: &mut Control) -> Result<bool> {
    let mut chars = s.chars();
    if !chars.next().is_some_and(nc_start) {
        return Ok(false);
    }
    for (i, x) in chars.enumerate() {
        if i & 1023 == 0 {
            c.poll()?
        }
        if !(nc_start(x)
            || matches!(x, '-' | '.' | '0'..='9')
            || matches!(x as u32,0xb7|0x300..=0x36f|0x203f..=0x2040))
        {
            return Ok(false);
        }
    }
    Ok(true)
}
pub fn qname<'a>(s: &'a str, c: &mut Control) -> Result<(Option<&'a str>, &'a str)> {
    let mut p = s.split(':');
    let a = p.next().unwrap();
    let z = p.next();
    if !nc_name(a, c)? || p.next().is_some() {
        return Err("InvalidXml");
    }
    match z {
        Some(z) if nc_name(z, c)? => Ok((Some(a), z)),
        Some(_) => Err("InvalidXml"),
        None => Ok((None, a)),
    }
}
pub fn reference(s: &str, c: &mut Control) -> Result<char> {
    let x = match s {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "apos" => '\'',
        "quot" => '"',
        _ => {
            if let Some(n) = s.strip_prefix('#') {
                let (n, radix) = if let Some(n) = n.strip_prefix('x') {
                    (n, 16)
                } else {
                    (n, 10)
                };
                if n.is_empty() {
                    return Err("InvalidXml");
                }
                let mut value = 0u32;
                for (i, b) in n.bytes().enumerate() {
                    if i & 4095 == 0 {
                        c.poll()?
                    }
                    let d = match b {
                        b'0'..=b'9' => (b - b'0') as u32,
                        b'a'..=b'f' if radix == 16 => (b - b'a' + 10) as u32,
                        b'A'..=b'F' if radix == 16 => (b - b'A' + 10) as u32,
                        _ => return Err("InvalidXml"),
                    };
                    value = value
                        .checked_mul(radix)
                        .and_then(|v| v.checked_add(d))
                        .ok_or("InvalidXml")?;
                }
                char::from_u32(value).ok_or("InvalidXml")?
            } else {
                if !nc_name(s, c)? {
                    return Err("InvalidXml");
                }
                return Err("UnsupportedXmlEntity");
            }
        }
    };
    if !legal(x) {
        return Err("InvalidXml");
    }
    Ok(x)
}
pub fn cooked_each(
    s: &str,
    attribute: bool,
    refs: bool,
    c: &mut Control,
    mut emit: impl FnMut(char) -> Result<()>,
) -> Result<()> {
    let mut it = s.char_indices().peekable();
    while let Some((i, x)) = it.next() {
        if i % 4096 < 4 {
            c.poll()?;
        }
        if !legal(x) {
            return Err("InvalidXml");
        }
        if attribute && x == '<' {
            return Err("InvalidXml");
        }
        if refs && x == '&' {
            let start = i + 1;
            let mut end = None;
            for (j, x) in it.by_ref() {
                if j % 4096 < 4 {
                    c.poll()?;
                }
                if x == ';' {
                    end = Some(j);
                    break;
                }
                if x == '<' || x == '&' {
                    return Err("InvalidXml");
                }
            }
            emit(reference(&s[start..end.ok_or("InvalidXml")?], c)?)?;
        } else {
            let x = if x == '\r' {
                if it.peek().is_some_and(|(_, x)| *x == '\n') {
                    it.next();
                }
                '\n'
            } else {
                x
            };
            emit(if attribute && matches!(x, '\t' | '\n' | '\r') {
                ' '
            } else {
                x
            })?;
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug)]
pub struct Attr {
    pub span: Span,
    pub name: Span,
    pub quoted: Span,
    pub value: Span,
}
pub fn attrs(s: &str, p: Span, c: &mut Control) -> Result<(Span, Vec<Attr>)> {
    let b = s.as_bytes();
    let mut i = p.start;
    while i < p.end && !is_s(b[i]) {
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
    }
    let name = Span {
        start: p.start,
        end: i,
    };
    qname(&s[name.start..name.end], c)?;
    let mut out = Vec::new();
    while i < p.end {
        if !is_s(b[i]) {
            return Err("InvalidXml");
        }
        while i < p.end && is_s(b[i]) {
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
        if i == p.end {
            break;
        }
        let a = i;
        while i < p.end && !is_s(b[i]) && b[i] != b'=' {
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
        let n = Span { start: a, end: i };
        qname(&s[a..i], c)?;
        while i < p.end && is_s(b[i]) {
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
        if b.get(i) != Some(&b'=') {
            return Err("InvalidXml");
        }
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
        while i < p.end && is_s(b[i]) {
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
        let q = i;
        let quote = *b.get(i).ok_or("InvalidXml")?;
        if !matches!(quote, b'\'' | b'"') {
            return Err("InvalidXml");
        }
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
        let value_start = i;
        while i < p.end && b[i] != quote {
            if i & 4095 == 0 {
                c.poll()?;
            }
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
        if i == p.end {
            return Err("InvalidXml");
        }
        let value = Span {
            start: value_start,
            end: i,
        };
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
        reserve(&mut out, 1, c)?;
        out.push(Attr {
            span: Span { start: a, end: i },
            name: n,
            quoted: Span { start: q, end: i },
            value,
        });
    }
    Ok((name, out))
}
pub fn declaration(s: &str, p: Span, c: &mut Control) -> Result<()> {
    let (n, a) = attrs(s, p, c)?;
    if &s[n.start..n.end] != "xml" || a.is_empty() {
        return Err("InvalidXml");
    }
    let mut rank = 0;
    let mut version = None;
    let mut encoding = None;
    let allocated = a.capacity() * std::mem::size_of::<Attr>();
    for a in a {
        let name = &s[a.name.start..a.name.end];
        let v = &s[a.value.start..a.value.end];
        let next = match name {
            "version" => 1,
            "encoding" => 2,
            "standalone" => 3,
            _ => return Err("InvalidXml"),
        };
        if next <= rank || (rank == 0 && next != 1) {
            return Err("InvalidXml");
        }
        rank = next;
        match name {
            "version" => {
                let mut iter = v.bytes();
                if iter.next() != Some(b'1')
                    || iter.next() != Some(b'.')
                    || v.len() < 3
                    || !ascii_all(&v.as_bytes()[2..], c, u8::is_ascii_digit)?
                {
                    return Err("InvalidXml");
                }
                version = Some(v)
            }
            "encoding" => {
                if !v.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                    || !ascii_all(v.as_bytes(), c, |x| {
                        x.is_ascii_alphanumeric() || matches!(*x, b'.' | b'_' | b'-')
                    })?
                {
                    return Err("InvalidXml");
                }
                encoding = Some(v)
            }
            "standalone" if !matches!(v, "yes" | "no") => return Err("InvalidXml"),
            _ => {}
        }
    }
    c.release(allocated);
    if version != Some("1.0") {
        return Err("UnsupportedXmlVersion");
    }
    if encoding.is_some_and(|x| !x.eq_ignore_ascii_case("utf-8")) {
        return Err("UnsupportedXmlEncoding");
    }
    Ok(())
}
pub fn doctype(s: &str, p: Span, c: &mut Control) -> Result<()> {
    let b = s.as_bytes();
    let mut i = p.start;
    if i == p.end || !is_s(b[i]) {
        return Err("InvalidXml");
    }
    while i < p.end && is_s(b[i]) {
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
    }
    let a = i;
    while i < p.end && !is_s(b[i]) && b[i] != b'[' {
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
    }
    qname(&s[a..i], c)?;
    let had_space = i < p.end && is_s(b[i]);
    while i < p.end && is_s(b[i]) {
        if i & 4095 == 0 {
            c.poll()?;
        }
        i += 1;
    }
    if i < p.end && b[i] != b'[' {
        if !had_space {
            return Err("InvalidXml");
        }
        let public = if s[i..p.end].starts_with("SYSTEM") {
            i += 6;
            false
        } else if s[i..p.end].starts_with("PUBLIC") {
            i += 6;
            true
        } else {
            return Err("InvalidXml");
        };
        for n in 0..if public { 2 } else { 1 } {
            if i == p.end || !is_s(b[i]) {
                return Err("InvalidXml");
            }
            while i < p.end && is_s(b[i]) {
                if i & 4095 == 0 {
                    c.poll()?;
                }
                i += 1;
            }
            let quote = *b.get(i).ok_or("InvalidXml")?;
            if !matches!(quote, b'\'' | b'"') {
                return Err("InvalidXml");
            }
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
            while i < p.end && b[i] != quote {
                if public
                    && n == 0
                    && !matches!(b[i],b' '|b'\r'|b'\n'|b'a'..=b'z'|b'A'..=b'Z'|b'0'..=b'9'|b'-'|b'\''|b'('|b')'|b'+'|b','|b'.'|b'/'|b':'|b'='|b'?'|b';'|b'!'|b'*'|b'#'|b'@'|b'$'|b'_'|b'%')
                {
                    return Err("InvalidXml");
                }
                if i & 4095 == 0 {
                    c.poll()?;
                }
                i += 1;
            }
            if i == p.end {
                return Err("InvalidXml");
            }
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
        while i < p.end && is_s(b[i]) {
            if i & 4095 == 0 {
                c.poll()?;
            }
            i += 1;
        }
    }
    if i < p.end {
        if b[i] != b'[' {
            return Err("InvalidXml");
        }
        let mut e = p.end;
        while e > i && is_s(b[e - 1]) {
            e -= 1;
        }
        if b[e - 1] != b']' {
            return Err("InvalidXml");
        } /* subset intentionally opaque */
    }
    Ok(())
}

fn ascii_all(bytes: &[u8], c: &mut Control, predicate: impl Fn(&u8) -> bool) -> Result<bool> {
    for chunk in bytes.chunks(4096) {
        c.poll()?;
        if !chunk.iter().all(&predicate) {
            return Ok(false);
        }
    }
    Ok(true)
}
