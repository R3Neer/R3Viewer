//! Closed graph validation shared by producer and host; no parser/IO callbacks.
use crate::{
    document_budget::{ARENA_LIMIT, ArenaVec, TRANSPORT_RESERVE},
    document_model::*,
    document_obsidian::{Control, WikiKind, validate_wiki},
};
fn invalid() -> Error {
    Error::new(ErrorKind::InvalidContext)
}
fn require(ok: bool) -> Result<()> {
    if ok { Ok(()) } else { Err(invalid()) }
}
fn get<T>(slice: &[T], id: u32) -> Result<&T> {
    slice.get(id as usize).ok_or_else(invalid)
}
fn range(first: u32, count: u32, len: usize) -> Result<std::ops::Range<usize>> {
    let first = first as usize;
    let end = first.checked_add(count as usize).ok_or_else(invalid)?;
    require(first <= end && end <= len)?;
    Ok(first..end)
}
fn optional_text(c: &Candidate, id: u32, ctl: &Control<'_>, pool: &str) -> Result<()> {
    if id != NONE {
        text(c, id, ctl, pool)?;
    }
    Ok(())
}
fn text<'a>(c: &'a Candidate, id: u32, ctl: &Control<'_>, pool: &'a str) -> Result<&'a str> {
    ctl.poll()?;
    let t = get(&c.texts, id)?;
    match t.kind() {
        0 => c.source.range(t.start(), t.end()),
        1 => pool
            .get(t.start() as usize..t.end() as usize)
            .ok_or_else(invalid),
        _ => Err(invalid()),
    }
}
fn raw(c: &Candidate, id: u32) -> Result<&TextRef> {
    let t = get(&c.texts, id)?;
    require(t.kind() == 0)?;
    let o = get(&c.origins, t.origin())?;
    require(o.relation() == 0 && o.start() == t.start() && o.end() == t.end())?;
    Ok(t)
}
fn origin(c: &Candidate, id: u32) -> Result<&Origin> {
    get(&c.origins, id)
}
fn contains(outer: &Origin, inner: &Origin) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
fn flow(kind: u32) -> bool {
    matches!(kind,1..=5|7|10..=14)
}
fn has_domain(kind: u32) -> bool {
    matches!(kind, 1 | 2 | 6 | 9 | 10 | 13 | 14)
}
fn zero(words: &mut [u32], ctl: &Control<'_>) -> Result<()> {
    for chunk in words.chunks_mut(1024) {
        ctl.poll()?;
        chunk.fill(0);
    }
    Ok(())
}
fn use_once(words: &mut [u32], offset: usize, id: u32) -> Result<()> {
    let at = words.get_mut(offset + id as usize).ok_or_else(invalid)?;
    require(*at == 0)?;
    *at = 1;
    Ok(())
}
/// Minimum final host admission. The actual allocation ledger remains authoritative.
pub(crate) fn preflight(counts: [u32; 11], old_bytes: usize) -> Result<(usize, usize)> {
    let k = counts[..10]
        .iter()
        .try_fold(0usize, |n, &x| n.checked_add(x as usize))
        .ok_or_else(|| Error::new(ErrorKind::RecordLimit))?;
    if counts[0] == 0 || k > RECORD_LIMIT {
        return Err(Error::new(ErrorKind::RecordLimit));
    }
    let strides = [64usize, 24, 16, 64, 40, 64, 80, 16, 16, 24];
    let w = counts[..10]
        .iter()
        .zip(strides)
        .try_fold(counts[10] as usize, |n, (&x, s)| {
            n.checked_add((x as usize).checked_mul(s)?)
        })
        .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
    let scratch = k
        .checked_mul(16)
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
    let total = w
        .checked_add(scratch)
        .and_then(|n| n.checked_add(TRANSPORT_RESERVE))
        .and_then(|n| n.checked_add(old_bytes))
        .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
    if total > ARENA_LIMIT {
        return Err(Error::new(ErrorKind::ArenaLimit));
    }
    Ok((w, scratch))
}
pub(crate) fn validate(c: &Candidate, ctl: &Control<'_>) -> Result<u32> {
    ctl.poll()?;
    let (_, scratch_bytes) = preflight(c.counts(), 0)?;
    require(c.attributes.is_empty() && c.decoded.len() <= ARENA_LIMIT)?;
    let pool = crate::document_utf8::checked(&c.decoded, ctl)?;
    let b = c.blocks.len();
    let d = c.domains.len();
    let r = c.runs.len();
    let s = c.spans.len();
    let l = c.links.len();
    let g = c.resources.len();
    let mut scratch = ArenaVec::<u32>::new(c.arena.clone());
    scratch.reserve_exact(scratch_bytes / 4)?;
    while scratch.len() < scratch_bytes / 4 {
        ctl.poll()?;
        let n = (scratch_bytes / 4 - scratch.len()).min(1024);
        scratch.extend_from_slice(&[0; 1024][..n])?;
    }
    // depth B and span-depth S remain resident while all other scratch is reused.
    let sd = b;
    let bs = b + s;
    let rs = bs + b;
    let gs = rs + r;
    let ls = gs + g;
    let sl = ls + l;
    let dl = sl + s;
    require(dl + d <= scratch.len())?;
    for (id, o) in c.origins.iter().enumerate() {
        ctl.poll()?;
        require(o.relation() <= 1 && o.reserved() == 0)?;
        c.source
            .range(o.start(), o.end())
            .map_err(|e| e.at(9, id as u32, NONE))?;
    }
    for (id, t) in c.texts.iter().enumerate() {
        ctl.poll()?;
        let o = origin(c, t.origin())?;
        text(c, id as u32, ctl, pool)?;
        require(t.kind() <= 1)?;
        if t.kind() == 0 {
            require(o.relation() == 0 && o.start() == t.start() && o.end() == t.end())?;
        }
    }
    let root = &c.blocks[0];
    require(root.id() == 0 && root.kind() == 0 && root.parent() == NONE)?;
    let ro = origin(c, root.origin())?;
    require(ro.relation() == 0 && ro.start() == 0 && ro.end() as usize == c.source.len())?;
    let mut item_end = 0;
    let mut max_depth = 0;
    for (id, block) in c.blocks.iter().enumerate() {
        ctl.poll()?;
        require(
            block.id() == id as u32
                && (id == 0 || block.kind() != 0)
                && block.kind() <= 14
                && block.flags() == 0
                && block.words()[12..].iter().all(|&x| x == 0),
        )?;
        let bo = origin(c, block.origin())?;
        require(block.first_item() as usize == item_end)?;
        let items = range(block.first_item(), block.item_count(), c.items.len())?;
        item_end = items.end;
        let mut at = id as u32;
        let mut depth = 0u32;
        while at != 0 {
            ctl.poll()?;
            let here = get(&c.blocks, at)?;
            let parent = get(&c.blocks, here.parent())?;
            require(contains(
                origin(c, parent.origin())?,
                origin(c, here.origin())?,
            ))?;
            depth += 1;
            if depth > DEPTH_LIMIT {
                return Err(Error::new(ErrorKind::DepthLimit));
            }
            at = here.parent();
        }
        scratch[id] = depth;
        max_depth = max_depth.max(depth);
        if has_domain(block.kind()) {
            let domain = get(&c.domains, block.domain())?;
            require(domain.kind() == 0 && domain.owner() == id as u32)?;
        } else {
            require(block.domain() == NONE)?;
        }
        match block.kind() {
            0 | 1 | 3 | 11 | 13 => {
                require([block.aux0(), block.aux1(), block.aux2(), block.aux3()] == [0; 4])?
            }
            2 => require(
                (1..=6).contains(&block.aux0())
                    && block.aux1() == NONE
                    && block.aux2() == 0
                    && block.aux3() == 0,
            )?,
            4 => {
                require(
                    c.profile == Profile::Extended
                        && !text(c, block.aux0(), ctl, pool)?.is_empty()
                        && block.aux2() <= 2,
                )?;
                let mo = origin(c, block.aux3())?;
                let marker = crate::document_parser::marker(
                    c.source.text(),
                    bo.start() as usize..bo.end() as usize,
                    ctl,
                )?
                .ok_or_else(invalid)?;
                require(
                    mo.relation() == 0
                        && mo.start() as usize == marker.marker.start
                        && mo.end() as usize == marker.marker.end
                        && block.aux2() == marker.fold,
                )?;
                require(crate::document_obsidian::equal(
                    text(c, block.aux0(), ctl, pool)?.as_bytes(),
                    c.source.text()[marker.kind].as_bytes(),
                    ctl,
                )?)?;
                if block.aux1() != NONE {
                    let title = get(&c.domains, block.aux1())?;
                    require(title.kind() == 2 && title.owner() == id as u32)?;
                }
            }
            5 => require(
                block.aux0() <= 1
                    && (block.aux0() == 1 || block.aux1() == 0)
                    && block.aux2() <= 1
                    && block.aux3() == 0,
            )?,
            6 => require(
                block.aux0() <= 2
                    && (c.profile == Profile::Extended || block.aux0() == 0)
                    && block.aux1() == 0
                    && block.aux2() == 0
                    && block.aux3() == 0,
            )?,
            7 => require(
                c.profile == Profile::Extended
                    && block.aux0() > 0
                    && block.aux0() != NONE
                    && block.aux1() == 0
                    && block.aux2() == 0
                    && block.aux3() == 0,
            )?,
            8 => {
                require(
                    block.aux0() <= 1
                        && block.aux1() == 0
                        && block.aux2() == 0
                        && block.aux3() == 0,
                )?;
                let table = get(&c.blocks, block.parent())?;
                require(table.kind() == 7 && block.item_count() == table.aux0())?;
            }
            9 => require(
                block.aux0() <= 3 && block.aux1() == 0 && block.aux2() == 0 && block.aux3() == 0,
            )?,
            10 => {
                require(block.aux0() <= 1)?;
                optional_text(c, block.aux1(), ctl, pool)?;
                optional_text(c, block.aux2(), ctl, pool)?;
                if block.aux3() != NONE {
                    let resource = get(&c.resources, block.aux3())?;
                    require(
                        resource.kind() == 1
                            && resource.owner_run() == NONE
                            && block.aux0() == 1
                            && text(c, block.aux2(), ctl, pool)? == "mermaid",
                    )?;
                    use_once(&mut scratch, gs, block.aux3())?;
                }
            }
            12 => {
                require(
                    c.profile == Profile::Extended
                        && block.aux1() == 0
                        && block.aux2() == 0
                        && block.aux3() == 0,
                )?;
                text(c, block.aux0(), ctl, pool)?;
            }
            14 => {
                require(
                    c.profile == Profile::Extended
                        && block.parent() == 0
                        && block.aux0() == 0
                        && block.aux1() == 0
                        && block.aux2() == 0
                        && block.aux3() == 0,
                )?;
                let token = c.source.range(bo.start(), bo.end())?;
                require(
                    bo.start() == 0 && (token.starts_with("---\n") || token.starts_with("---\r\n")),
                )?;
            }
            _ => return Err(invalid()),
        }
        let mut next_run = 0;
        let mut row = 0;
        for (ordinal, item) in c.items[items].iter().enumerate() {
            ctl.poll()?;
            require(item.parent_block() == id as u32 && item.ordinal() == ordinal as u32)?;
            match item.kind() {
                0 => {
                    let child = get(&c.blocks, item.target())?;
                    require(child.parent() == id as u32 && item.target() != 0)?;
                    use_once(&mut scratch, bs, item.target())?;
                    require(match block.kind() {
                        0 | 3 | 4 | 6 | 12 => flow(child.kind()),
                        5 => child.kind() == 6,
                        7 => child.kind() == 8,
                        8 => child.kind() == 9,
                        _ => false,
                    })?;
                    if child.kind() == 14 {
                        require(block.kind() == 0 && ordinal == 0)?;
                    }
                    if block.kind() == 7 {
                        require(child.aux0() == u32::from(row == 0))?;
                        row += 1;
                    }
                }
                1 => {
                    require(has_domain(block.kind()))?;
                    let run = get(&c.runs, item.target())?;
                    require(run.domain() == block.domain() && run.ordinal() == next_run)?;
                    next_run += 1;
                    use_once(&mut scratch, rs, item.target())?;
                }
                _ => return Err(invalid()),
            }
        }
        if block.kind() == 7 {
            require(row > 0)?;
        }
        if has_domain(block.kind()) {
            require(next_run == get(&c.domains, block.domain())?.run_count())?;
        }
        let _ = bo;
    }
    require(item_end == c.items.len() && scratch[bs + 1..bs + b].iter().all(|&x| x == 1))?;
    let mut run_end = 0;
    for (id, domain) in c.domains.iter().enumerate() {
        ctl.poll()?;
        require(
            domain.id() == id as u32
                && domain.kind() <= 2
                && domain.words()[5] == 0
                && domain.first_run() as usize == run_end,
        )?;
        let runs = range(domain.first_run(), domain.run_count(), r)?;
        run_end = runs.end;
        match domain.kind() {
            0 => {
                let block = get(&c.blocks, domain.owner())?;
                require(has_domain(block.kind()) && block.domain() == id as u32)?;
            }
            1 => {
                let resource = get(&c.resources, domain.owner())?;
                require(matches!(resource.kind(), 0 | 4) && resource.alt_domain() == id as u32)?;
            }
            2 => {
                let block = get(&c.blocks, domain.owner())?;
                require(block.kind() == 4 && block.aux1() == id as u32)?;
            }
            _ => return Err(invalid()),
        }
        for (ordinal, run) in c.runs[runs].iter().enumerate() {
            ctl.poll()?;
            let runid = domain.first_run() + ordinal as u32;
            require(
                run.id() == runid
                    && run.domain() == id as u32
                    && run.ordinal() == ordinal as u32
                    && run.kind() <= 7
                    && run.styles() & !15 == 0
                    && run.aux0() == 0
                    && run.aux1() == 0
                    && run.words()[11..].iter().all(|&x| x == 0),
            )?;
            origin(c, run.origin())?;
            if run.kind() == 4 {
                require(c.profile == Profile::Extended)?;
            }
            require(scratch[rs + runid as usize] == u32::from(domain.kind() == 0))?;
            if matches!(run.kind(), 2 | 3 | 5) {
                require(run.text() == NONE)?;
            } else {
                text(c, run.text(), ctl, pool)?;
            }
            if run.kind() == 5 {
                require(domain.kind() == 0)?;
                let res = get(&c.resources, run.resource())?;
                require(res.kind() != 1 && res.owner_run() == runid)?;
                use_once(&mut scratch, gs, run.resource())?;
            } else {
                require(run.resource() == NONE)?;
            }
            if domain.kind() != 0 {
                require(run.kind() <= 4)?;
            }
            if domain.kind() == 0 {
                match get(&c.blocks, domain.owner())?.kind() {
                    10 | 14 => require(run.kind() == 6 && run.link() == NONE)?,
                    13 => require(run.kind() == 7 && run.link() == NONE)?,
                    _ => require(!matches!(run.kind(), 6 | 7))?,
                }
            }
        }
    }
    require(run_end == r && scratch[gs..gs + g].iter().all(|&x| x == 1))?;
    let mut previous: Option<(u32, u32, u32, u32, u32, u32)> = None;
    for (id, span) in c.spans.iter().enumerate() {
        ctl.poll()?;
        require(
            span.id() == id as u32 && span.kind() <= 3 && span.words()[8..].iter().all(|&x| x == 0),
        )?;
        let domain = get(&c.domains, span.domain())?;
        require(
            span.start_ordinal() <= span.end_ordinal() && span.end_ordinal() <= domain.run_count(),
        )?;
        let so = origin(c, span.origin())?;
        if domain.kind() == 0 {
            require(!matches!(
                get(&c.blocks, domain.owner())?.kind(),
                10 | 13 | 14
            ))?;
        }
        if span.kind() == 2 {
            require(c.profile == Profile::Extended)?;
        }
        let depth = if span.parent_span() == NONE {
            1
        } else {
            require(span.parent_span() < id as u32)?;
            let parent = get(&c.spans, span.parent_span())?;
            require(
                parent.domain() == span.domain()
                    && parent.start_ordinal() <= span.start_ordinal()
                    && span.end_ordinal() <= parent.end_ordinal()
                    && contains(origin(c, parent.origin())?, so),
            )?;
            scratch[sd + span.parent_span() as usize] + 1
        };
        if depth > DEPTH_LIMIT {
            return Err(Error::new(ErrorKind::DepthLimit));
        }
        scratch[sd + id] = depth;
        let last = if span.parent_span() == NONE {
            dl + span.domain() as usize
        } else {
            sl + span.parent_span() as usize
        };
        require(scratch[last] <= span.start_ordinal())?;
        scratch[last] = span.end_ordinal();
        let order = (
            span.domain(),
            span.start_ordinal(),
            u32::MAX - span.end_ordinal(),
            depth,
            so.start(),
            so.end(),
        );
        if let Some(p) = previous {
            require(p <= order)?;
        }
        previous = Some(order);
        if span.kind() == 3 {
            let link = get(&c.links, span.link())?;
            require(
                link.label_domain() == span.domain()
                    && link.label_start() == span.start_ordinal()
                    && link.label_count() == span.end_ordinal() - span.start_ordinal(),
            )?;
            use_once(&mut scratch, ls, span.link())?;
            let mut at = span.parent_span();
            while at != NONE {
                ctl.poll()?;
                let ancestor = get(&c.spans, at)?;
                require(ancestor.kind() != 3)?;
                at = ancestor.parent_span();
            }
            for run in &c.runs[range(
                domain.first_run() + span.start_ordinal(),
                span.end_ordinal() - span.start_ordinal(),
                r,
            )?] {
                ctl.poll()?;
                require(run.link() == span.link())?;
            }
        } else {
            require(span.link() == NONE)?;
        }
    }
    require(scratch[ls..ls + l].iter().all(|&x| x == 1))?;
    for (id, link) in c.links.iter().enumerate() {
        ctl.poll()?;
        require(
            link.id() == id as u32
                && link.kind() <= 9
                && link.words()[12..].iter().all(|&x| x == 0)
                && if link.kind() == 9 {
                    link.flags() <= 1
                } else {
                    link.flags() == 0
                },
        )?;
        text(c, link.target_decoded(), ctl, pool)?;
        optional_text(c, link.title_decoded(), ctl, pool)?;
        optional_text(c, link.reference_label(), ctl, pool)?;
        raw(c, link.raw_token())?;
        let o = origin(c, link.origin())?;
        let t = get(&c.texts, link.raw_token())?;
        require(o.relation() == 0 && o.start() == t.start() && o.end() == t.end())?;
        if link.target_origin() != NONE {
            origin(c, link.target_origin())?;
        }
        if link.kind() == 9 {
            require(c.profile == Profile::Extended)?;
            validate_wiki(
                c,
                c.profile,
                link.label_domain(),
                link.target_decoded(),
                link.raw_token(),
                WikiKind::Link {
                    pothole: link.flags() == 1,
                },
                ctl,
                pool,
            )?;
        }
    }
    for run in c.runs.iter() {
        ctl.poll()?;
        if run.link() != NONE {
            let link = get(&c.links, run.link())?;
            require(
                link.label_domain() == run.domain()
                    && link.label_start() <= run.ordinal()
                    && run.ordinal() < link.label_start() + link.label_count(),
            )?;
        }
    }
    for (id, res) in c.resources.iter().enumerate() {
        ctl.poll()?;
        require(
            res.id() == id as u32
                && res.kind() <= 4
                && res.flags() == 0
                && res.aux0() == 0
                && res.aux1() == 0
                && res.words()[16..].iter().all(|&x| x == 0),
        )?;
        text(c, res.input_decoded(), ctl, pool)?;
        let t = raw(c, res.raw_token())?;
        let o = origin(c, res.origin())?;
        require(o.relation() == 0 && o.start() == t.start() && o.end() == t.end())?;
        for (kind, value) in [
            (res.width_kind(), res.width_value()),
            (res.height_kind(), res.height_value()),
        ] {
            require((kind == 0 && value == 0) || (kind == 1 && value > 0 && value != NONE))?;
        }
        require(res.intent() == u32::from(matches!(res.kind(), 1 | 3)))?;
        if matches!(res.kind(), 0 | 4) {
            let alt = get(&c.domains, res.alt_domain())?;
            require(alt.kind() == 1 && alt.owner() == id as u32)?;
            optional_text(c, res.title_decoded(), ctl, pool)?;
        } else {
            require(
                res.alt_domain() == NONE
                    && res.title_decoded() == NONE
                    && res.width_kind() == 0
                    && res.height_kind() == 0,
            )?;
        }
        if res.kind() == 1 {
            require(res.owner_run() == NONE)?;
        } else {
            let run = get(&c.runs, res.owner_run())?;
            require(run.kind() == 5 && run.resource() == id as u32)?;
        }
        if matches!(res.kind(), 2..=4) {
            require(c.profile == Profile::Extended)?;
        }
        if res.kind() == 4 {
            let run = get(&c.runs, res.owner_run())?;
            validate_wiki(
                c,
                c.profile,
                run.domain(),
                res.input_decoded(),
                res.raw_token(),
                WikiKind::Embed,
                ctl,
                pool,
            )?;
            let (w, h) = crate::document_obsidian::image_dimensions(
                c.source.range(t.start(), t.end())?,
                ctl,
            )?;
            require(
                res.width_value() == w
                    && res.height_value() == h
                    && res.width_kind() == u32::from(w != 0)
                    && res.height_kind() == u32::from(h != 0),
            )?;
        }
    }
    // Reuse graph counters as ordinal-boundary deltas. Domain separators allow empty domains.
    let n = r + d;
    let start = b + s;
    require(start + 4 * n <= scratch.len())?;
    zero(&mut scratch[start..], ctl)?;
    for span in c.spans.iter() {
        ctl.poll()?;
        let domain = get(&c.domains, span.domain())?;
        let first = domain.first_run() as usize + span.domain() as usize;
        let a = first + span.start_ordinal() as usize;
        let z = first + span.end_ordinal() as usize;
        if span.kind() < 3 {
            let lane = start + span.kind() as usize * n;
            scratch[lane + a] = scratch[lane + a].wrapping_add(1);
            scratch[lane + z] = scratch[lane + z].wrapping_sub(1);
        }
        let lane = start + 3 * n;
        scratch[lane + a] = scratch[lane + a].wrapping_add(1);
        scratch[lane + z] = scratch[lane + z].wrapping_sub(1);
    }
    for domain in c.domains.iter() {
        let first = domain.first_run() as usize + domain.id() as usize;
        let mut active = [0u32; 4];
        for ordinal in 0..=domain.run_count() as usize {
            ctl.poll()?;
            let at = first + ordinal;
            for lane in 0..4 {
                active[lane] = active[lane].wrapping_add(scratch[start + lane * n + at]);
                require(active[lane] <= DEPTH_LIMIT)?;
            }
            if ordinal < domain.run_count() as usize {
                let run = &c.runs[domain.first_run() as usize + ordinal];
                let mut styles = 0;
                for (lane, &value) in active[..3].iter().enumerate() {
                    if value > 0 {
                        styles |= 1 << lane;
                    }
                }
                if matches!(run.kind(), 1 | 6) {
                    styles |= 8;
                }
                require(run.styles() == styles)?;
                scratch[start + 3 * n + at] = active[3];
            } else {
                require(active == [0; 4])?;
            }
        }
    }
    for domain in c.domains.iter() {
        ctl.poll()?;
        let base = match domain.kind() {
            0 | 2 => scratch[domain.owner() as usize],
            1 => {
                let res = get(&c.resources, domain.owner())?;
                let owner = get(&c.runs, res.owner_run())?;
                let parent = get(&c.domains, owner.domain())?;
                require(parent.kind() == 0)?;
                scratch[parent.owner() as usize]
                    + scratch[start
                        + 3 * n
                        + parent.first_run() as usize
                        + parent.id() as usize
                        + owner.ordinal() as usize]
            }
            _ => return Err(invalid()),
        };
        max_depth = max_depth.max(base);
        for run in &c.runs[range(domain.first_run(), domain.run_count(), r)?] {
            ctl.poll()?;
            let depth = base
                + scratch[start
                    + 3 * n
                    + domain.first_run() as usize
                    + domain.id() as usize
                    + run.ordinal() as usize];
            max_depth = max_depth.max(depth);
        }
    }
    for (id, span) in c.spans.iter().enumerate() {
        ctl.poll()?;
        let domain = get(&c.domains, span.domain())?;
        let base = if domain.kind() != 1 {
            scratch[domain.owner() as usize]
        } else {
            let res = get(&c.resources, domain.owner())?;
            let owner = get(&c.runs, res.owner_run())?;
            let parent = get(&c.domains, owner.domain())?;
            scratch[parent.owner() as usize]
                + scratch[start
                    + 3 * n
                    + parent.first_run() as usize
                    + parent.id() as usize
                    + owner.ordinal() as usize]
        };
        max_depth = max_depth.max(base + scratch[sd + id]);
    }
    if max_depth > DEPTH_LIMIT {
        return Err(Error::new(ErrorKind::DepthLimit));
    }
    ctl.poll()?;
    Ok(max_depth)
}

/// Only this module can construct the proof. No mutable payload escape exists.
pub(crate) struct ValidatedCandidate {
    candidate: Candidate,
}
impl std::ops::Deref for ValidatedCandidate {
    type Target = Candidate;
    fn deref(&self) -> &Candidate {
        &self.candidate
    }
}
impl ValidatedCandidate {
    pub(crate) fn into_inner(self) -> Candidate {
        self.candidate
    }
}
pub(crate) fn quarantine(
    candidate: Candidate,
    declared_depth: u32,
    ctl: &Control<'_>,
) -> Result<ValidatedCandidate> {
    let actual = validate(&candidate, ctl)?;
    if actual != declared_depth {
        return Err(Error::new(ErrorKind::InvalidIpc));
    }
    Ok(ValidatedCandidate { candidate })
}

#[cfg(test)]
mod pool_tests {
    use super::*;
    use crate::{
        document_budget::ActorArena, document_source::DocumentSource, input::SourceOrigin,
    };
    use std::{
        sync::{Arc, atomic::AtomicBool},
        time::{Duration, Instant},
    };
    #[test]
    fn overlapping_decoded_refs_share_one_utf8_pool_proof() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(String::new()),
                SourceOrigin::Literal,
                0,
                0,
                &cancel,
            )
            .unwrap(),
        );
        let mut c = Candidate::empty(source, Profile::Core, ActorArena::new());
        c.origins.push(Origin::from_words([0, 0, 0, 0])).unwrap();
        c.blocks
            .push(Block::from_words([
                0, NONE, 0, 0, NONE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]))
            .unwrap();
        let payload = vec![b'a'; 16 * 1024 * 1024];
        c.decoded.extend_from_slice(&payload).unwrap();
        drop(payload);
        c.texts.reserve_exact(50_000).unwrap();
        for _ in 0..50_000 {
            c.texts
                .push(TextRef::from_words([1, 0, 16 * 1024 * 1024, 0]))
                .unwrap();
        }
        assert_eq!(validate(&c, &ctl).unwrap(), 0);
        let proof = quarantine(c, 0, &ctl).unwrap();
        let document = Document::from_validated(proof);
        assert_eq!(document.text(49_999).unwrap().len(), 16 * 1024 * 1024);
    }
}
