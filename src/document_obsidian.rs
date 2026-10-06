//! OBS-PARSE01: only a checked unique cell-pipe escape is semantic adaptation.
use crate::document_model::{Candidate, Error, ErrorKind, NONE, Profile, Result};
use std::{
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
#[derive(Clone, Copy, Debug)]
pub(crate) enum WikiKind {
    Link { pothole: bool },
    Embed,
}
pub(crate) struct Control<'a> {
    pub cancel: &'a AtomicBool,
    pub deadline: Instant,
}
impl Control<'_> {
    pub fn poll(&self) -> Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(Error::new(ErrorKind::Cancelled))
        } else if Instant::now() > self.deadline {
            Err(Error::new(ErrorKind::Timeout))
        } else {
            Ok(())
        }
    }
}
pub(crate) struct Target<'a> {
    pub value: &'a str,
    pub adapted: bool,
    /// Pinned first-body-node starts at the second byte of an initial escaped backslash pair.
    pub first_body_loss: bool,
    pub token: Range<u32>,
}
fn invalid() -> Error {
    Error::new(ErrorKind::InvalidContext)
}
pub(crate) fn equal(a: &[u8], b: &[u8], control: &Control<'_>) -> Result<bool> {
    if a.len() != b.len() {
        control.poll()?;
        return Ok(false);
    }
    for (left, right) in a.chunks(1024).zip(b.chunks(1024)) {
        control.poll()?;
        if left != right {
            return Ok(false);
        }
    }
    control.poll()?;
    Ok(true)
}
/// Caller has validated profile/token/ranges and uses actual parser destination.
pub(crate) fn expected<'a>(
    source: &'a str,
    range: Range<u32>,
    profile: Profile,
    kind: WikiKind,
    cell: Option<Range<u32>>,
    original: Option<&str>,
    control: &Control<'_>,
) -> Result<Target<'a>> {
    control.poll()?;
    let token = source
        .get(range.start as usize..range.end as usize)
        .ok_or_else(invalid)?;
    let open = match kind {
        WikiKind::Link { .. } => "[[",
        WikiKind::Embed => "![[",
    };
    if !token.starts_with(open) || !token.ends_with("]]") || token.len() < open.len() + 2 {
        return Err(invalid());
    }
    if let Some(ref cell) = cell
        && (range.start < cell.start || range.end > cell.end)
    {
        return Err(invalid());
    }
    let inner = &token[open.len()..token.len() - 2];
    let mut pipe = None;
    for (block, bytes) in inner.as_bytes().chunks(1024).enumerate() {
        control.poll()?;
        if let Some(i) = bytes.iter().position(|&b| b == b'|') {
            pipe = Some(block * 1024 + i);
            break;
        }
    }
    if let WikiKind::Link { pothole } = kind
        && pothole != pipe.is_some()
    {
        return Err(invalid());
    }
    let end = pipe.unwrap_or(inner.len());
    let prefix = &inner[..end];
    // Source is semantic authority; prove the exact pinned upstream loss before restoring it.
    // firstpass sets begin_text=ix+1 for an initial escaped backslash; handle_wikilink
    // uses that first body node start. No generic unescape or mismatch fallback applies.
    let first_body_loss = profile == Profile::Extended && prefix.as_bytes().starts_with(b"\\\\");
    let parser_prefix = if first_body_loss {
        &prefix[1..]
    } else {
        prefix
    };
    if let Some(original) = original
        && !equal(parser_prefix.as_bytes(), original.as_bytes(), control)?
    {
        return Err(invalid());
    }
    let unique = end > 0
        && inner.as_bytes()[end - 1] == b'\\'
        && (end == 1 || inner.as_bytes()[end - 2] != b'\\');
    let adapted = profile == Profile::Extended && cell.is_some() && pipe.is_some() && unique;
    let value = if adapted {
        &prefix[..prefix.len() - 1]
    } else {
        prefix
    };
    control.poll()?;
    Ok(Target {
        value,
        adapted,
        first_body_loss,
        token: range,
    })
}
/// Derive cell exclusively from already validated typed ownership.
pub(crate) fn cell_for_domain(
    c: &Candidate,
    domain: u32,
    token: Range<u32>,
    control: &Control<'_>,
) -> Result<Option<Range<u32>>> {
    control.poll()?;
    let d = c.domains.get(domain as usize).ok_or_else(invalid)?;
    let owner = match d.kind() {
        0 => {
            let b = c.blocks.get(d.owner() as usize).ok_or_else(invalid)?;
            if b.domain() != domain {
                return Err(invalid());
            }
            d.owner()
        }
        1 => {
            let resource = c.resources.get(d.owner() as usize).ok_or_else(invalid)?;
            if resource.alt_domain() != domain {
                return Err(invalid());
            }
            let run = c
                .runs
                .get(resource.owner_run() as usize)
                .ok_or_else(invalid)?;
            if run.kind() != 5 || run.resource() != d.owner() {
                return Err(invalid());
            }
            let parent = c.domains.get(run.domain() as usize).ok_or_else(invalid)?;
            if parent.kind() != 0 {
                return Err(invalid());
            }
            let block = c.blocks.get(parent.owner() as usize).ok_or_else(invalid)?;
            if block.domain() != run.domain() {
                return Err(invalid());
            }
            parent.owner()
        }
        2 => {
            let b = c.blocks.get(d.owner() as usize).ok_or_else(invalid)?;
            if b.kind() != 4 || b.aux1() != domain {
                return Err(invalid());
            }
            d.owner()
        }
        _ => return Err(invalid()),
    };
    let mut at = owner;
    for _ in 0..=64 {
        control.poll()?;
        let b = c.blocks.get(at as usize).ok_or_else(invalid)?;
        if b.kind() == 9 {
            let origin = c.origins.get(b.origin() as usize).ok_or_else(invalid)?;
            let range = origin.start()..origin.end();
            if token.start < range.start || token.end > range.end {
                return Err(invalid());
            }
            return Ok(Some(range));
        }
        if b.parent() == NONE {
            if at != 0 {
                return Err(invalid());
            }
            return Ok(None);
        }
        at = b.parent();
    }
    Err(invalid())
}
/// Host reconstructs expectation without another original-target blob.
// Explicit checked graph fields keep caller ownership and pool proof visible.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_wiki(
    c: &Candidate,
    profile: Profile,
    domain: u32,
    target: u32,
    raw: u32,
    kind: WikiKind,
    control: &Control<'_>,
    pool: &str,
) -> Result<()> {
    let raw_ref = c.texts.get(raw as usize).ok_or_else(invalid)?;
    if raw_ref.kind() != 0 {
        return Err(invalid());
    }
    let range = raw_ref.start()..raw_ref.end();
    let cell = cell_for_domain(c, domain, range.clone(), control)?;
    let e = expected(c.source.text(), range, profile, kind, cell, None, control)?;
    let target_ref = c.texts.get(target as usize).ok_or_else(invalid)?;
    let value = match target_ref.kind() {
        0 => c.source.range(target_ref.start(), target_ref.end())?,
        1 => pool
            .get(target_ref.start() as usize..target_ref.end() as usize)
            .ok_or_else(invalid)?,
        _ => return Err(invalid()),
    };
    if !equal(value.as_bytes(), e.value.as_bytes(), control)? {
        return Err(invalid());
    }
    if e.adapted || e.first_body_loss {
        let t = c.texts.get(target as usize).ok_or_else(invalid)?;
        let o = c.origins.get(t.origin() as usize).ok_or_else(invalid)?;
        if t.kind() != 1
            || o.relation() != 1
            || o.start() != e.token.start
            || o.end() != e.token.end
        {
            return Err(invalid());
        }
    }
    Ok(())
}

/// Whole ASCII dimension aliases only; no trimming, parity, entities or partial numbers.
pub(crate) fn image_dimensions(token: &str, ctl: &Control<'_>) -> Result<(u32, u32)> {
    let mut pipe = None;
    for (base, chunk) in token.as_bytes().chunks(1024).enumerate() {
        ctl.poll()?;
        if let Some(at) = chunk.iter().position(|&c| c == b'|') {
            pipe = Some(base * 1024 + at);
            break;
        }
    }
    let Some(pipe) = pipe else {
        return Ok((0, 0));
    };
    if !token.ends_with("]]") {
        return Err(invalid());
    }
    let Some(alias) = token.get(pipe + 1..token.len() - 2) else {
        return Err(invalid());
    };
    if alias.len() > 21 {
        return Ok((0, 0));
    }
    fn number(s: &str) -> Option<u32> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        s.parse::<u32>().ok().filter(|&n| n > 0 && n != NONE)
    }
    let parsed = if let Some((w, h)) = alias.split_once('x') {
        number(w).zip(number(h))
    } else {
        number(alias).map(|w| (w, 0))
    };
    ctl.poll()?;
    Ok(parsed.unwrap_or((0, 0)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn unique_escape_requires_checked_context_and_preserves_interior_backslashes() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let s = "[[\\\\server\\share\\Nota\\|**Alias**]]";
        let range = 0..s.len() as u32;
        let r = expected(
            s,
            range.clone(),
            Profile::Extended,
            WikiKind::Link { pothole: true },
            Some(range.clone()),
            Some("\\server\\share\\Nota\\"),
            &ctl,
        )
        .unwrap();
        assert_eq!(r.value, "\\\\server\\share\\Nota");
        assert!(r.adapted);
        let outside = expected(
            s,
            range.clone(),
            Profile::Extended,
            WikiKind::Link { pothole: true },
            None,
            Some("\\server\\share\\Nota\\"),
            &ctl,
        )
        .unwrap();
        assert_eq!(outside.value, "\\\\server\\share\\Nota\\");
        assert!(!outside.adapted);
        assert_eq!(
            expected(
                s,
                range,
                Profile::Extended,
                WikiKind::Link { pothole: false },
                None,
                None,
                &ctl
            )
            .err()
            .unwrap()
            .kind,
            ErrorKind::InvalidContext
        );
    }
    #[test]
    fn repeated_escape_and_inconsistent_destination_never_receive_literal_fallback() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        for n in 2..=4 {
            let prefix = format!("Nota{}", "\\".repeat(n));
            let s = format!("[[{prefix}|Alias]]");
            let range = 0..s.len() as u32;
            let e = expected(
                &s,
                range.clone(),
                Profile::Extended,
                WikiKind::Link { pothole: true },
                Some(range),
                Some(&prefix),
                &ctl,
            )
            .unwrap();
            assert_eq!(e.value, prefix);
            assert!(!e.adapted);
        }
        let s = "[[Nota\\|Alias]]";
        assert_eq!(
            expected(
                s,
                0..s.len() as u32,
                Profile::Core,
                WikiKind::Link { pothole: true },
                None,
                Some("wrong"),
                &ctl
            )
            .err()
            .unwrap()
            .kind,
            ErrorKind::InvalidContext
        );
    }
}

#[cfg(test)]
mod first_body_loss_tests {
    use super::*;
    use crate::{
        document_budget::ActorArena, document_parser, document_source::DocumentSource,
        input::SourceOrigin,
    };
    use std::sync::Arc;
    use std::time::Duration;
    fn control(cancel: &AtomicBool) -> Control<'_> {
        Control {
            cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        }
    }
    #[test]
    fn proof_accepts_only_exact_one_initial_byte_loss() {
        let cancel = AtomicBool::new(false);
        let ctl = control(&cancel);
        for n in 2..=4 {
            let prefix = format!("{}server\\share\\Nota\\", "\\".repeat(n));
            let token = format!("[[{prefix}|Alias]]");
            let range = 0..token.len() as u32;
            let result = expected(
                &token,
                range.clone(),
                Profile::Extended,
                WikiKind::Link { pothole: true },
                Some(range.clone()),
                Some(&prefix[1..]),
                &ctl,
            )
            .unwrap();
            assert!(result.first_body_loss && result.adapted);
            assert_eq!(result.value, &prefix[..prefix.len() - 1]);
            for incorrect in [
                &prefix,
                &prefix[2..],
                "arbitrary",
                &prefix[1..prefix.len() - 1],
            ] {
                assert_eq!(
                    expected(
                        &token,
                        range.clone(),
                        Profile::Extended,
                        WikiKind::Link { pothole: true },
                        Some(range.clone()),
                        Some(incorrect),
                        &ctl
                    )
                    .err()
                    .unwrap()
                    .kind,
                    ErrorKind::InvalidContext
                );
            }
            assert_eq!(
                expected(
                    &token,
                    range,
                    Profile::Core,
                    WikiKind::Link { pothole: true },
                    None,
                    Some(&prefix[1..]),
                    &ctl
                )
                .err()
                .unwrap()
                .kind,
                ErrorKind::InvalidContext
            );
        }
    }
    #[test]
    fn trailing_repeated_escapes_remain_literal_after_initial_source_restore() {
        let cancel = AtomicBool::new(false);
        let ctl = control(&cancel);
        for n in 2..=4 {
            let prefix = format!("\\\\server\\share\\Nota{}", "\\".repeat(n));
            let token = format!("[[{prefix}|Alias]]");
            let range = 0..token.len() as u32;
            let result = expected(
                &token,
                range.clone(),
                Profile::Extended,
                WikiKind::Link { pothole: true },
                Some(range),
                Some(&prefix[1..]),
                &ctl,
            )
            .unwrap();
            assert!(result.first_body_loss);
            assert!(!result.adapted);
            assert_eq!(result.value, prefix);
        }
    }
    #[test]
    fn host_rejects_lost_prefix_and_forged_origin() {
        let cancel = AtomicBool::new(false);
        let ctl = control(&cancel);
        for mutation in 0..3 {
            let source = Arc::new(
                DocumentSource::new(
                    Arc::new("[[\\\\server\\share\\Nota|Alias]]".to_owned()),
                    SourceOrigin::Literal,
                    1,
                    1,
                    &cancel,
                )
                .unwrap(),
            );
            let mut c =
                document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
                    .unwrap();
            let target = c.links[0].target_decoded() as usize;
            let origin = c.texts[target].origin() as usize;
            match mutation {
                0 => c.texts[target].0[1] += 1,
                1 => c.origins[origin].0[0] = 0,
                _ => c.origins[origin].0[1] += 1,
            }
            assert_eq!(
                crate::document_validate::validate(&c, &ctl)
                    .unwrap_err()
                    .kind,
                ErrorKind::InvalidContext
            );
        }
    }
    #[test]
    fn core_code_and_html_never_receive_unc_repair() {
        let cancel = AtomicBool::new(false);
        let ctl = control(&cancel);
        for (profile, text) in [
            (Profile::Core, "[[\\\\server\\share\\Nota|Alias]]"),
            (
                Profile::Extended,
                "```text\n[[\\\\server\\share\\Nota|Alias]]\n```\n",
            ),
            (
                Profile::Extended,
                "<div>[[\\\\server\\share\\Nota|Alias]]</div>\n",
            ),
        ] {
            let source = Arc::new(
                DocumentSource::new(
                    Arc::new(text.to_owned()),
                    SourceOrigin::Literal,
                    1,
                    1,
                    &cancel,
                )
                .unwrap(),
            );
            let c = document_parser::project(source, profile, ActorArena::new(), &ctl).unwrap();
            assert!(c.links.is_empty() && c.resources.is_empty());
            assert_eq!(c.source.text(), text);
        }
    }
}
