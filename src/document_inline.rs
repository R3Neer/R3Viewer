//! Complete document output. The caller receives a String only after both passes succeed.
use crate::{
    config::{PresentationConfig, Role},
    document_consumer_budget::{ChargedVec, ConsumerBudget},
    document_model::{Document, NONE},
    document_text::{
        DocumentWork, Error, IndexProfile, SegmentedIndex, SegmentedText, TextRefIndex,
    },
};
use ratatui::style::Color;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const MAX_OUTPUT: usize = 256 * 1024 * 1024;
#[derive(Clone, Copy)]
pub(crate) struct Geometry {
    pub columns: u16,
    pub line_numbers: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Document,
    Source,
}
#[derive(Debug)]
pub(crate) struct PreparedOutput {
    pub content: String,
    pub mode: Mode,
    pub diagnostic: Option<&'static str>,
}
/// The same admitted original is retained through fallback. No reopening or Source index.
pub(crate) fn render(
    document: Arc<Document>,
    columns: u16,
    config: &PresentationConfig,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<PreparedOutput, Error> {
    match prepare_with_config(document.clone(), columns, config, cancel, limit) {
        Ok(content) => Ok(PreparedOutput {
            content,
            mode: Mode::Document,
            diagnostic: None,
        }),
        Err(Error::Cancelled) => Err(Error::Cancelled),
        Err(reason) => {
            let source = document.source().original_handle();
            let content = source_fallback(&source, config.line_numbers, cancel, limit)?;
            let diagnostic = match reason {
                Error::Budget => {
                    "no se pudo representar Documento; se muestra Fuente completa: presupuesto"
                }
                Error::Work => {
                    "no se pudo representar Documento; se muestra Fuente completa: trabajo"
                }
                Error::Invalid => {
                    "no se pudo representar Documento; se muestra Fuente completa: posición"
                }
                Error::UnsupportedRun(_) => {
                    "no se pudo representar Documento; se muestra Fuente completa: recurso"
                }
                Error::Cancelled => return Err(Error::Cancelled),
            };
            Ok(PreparedOutput {
                content,
                mode: Mode::Source,
                diagnostic: Some(diagnostic),
            })
        }
    }
}
pub(crate) fn source_fallback(
    source: &str,
    numbers: bool,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, Error> {
    if !numbers {
        return crate::document_inline_source::prepare(source, cancel, limit).map_err(
            |e| match e {
                crate::document_inline_source::Error::Cancelled => Error::Cancelled,
                _ => Error::Budget,
            },
        );
    }
    let mut output = Output {
        value: None,
        bytes: 0,
        limit: limit.min(MAX_OUTPUT),
        geometry: Geometry {
            columns: 0,
            line_numbers: true,
        },
        line: 1,
        column: 0,
        has_content: false,
        has_semantic: false,
        cancel,
        config: None,
        style: DisplayStyle::PLAIN,
    };
    source_pass(source, &mut output)?;
    output.check()?;
    let needed = output.bytes;
    let mut candidate = String::new();
    candidate
        .try_reserve_exact(needed)
        .map_err(|_| Error::Budget)?;
    if candidate.capacity() > output.limit {
        return Err(Error::Budget);
    }
    output.value = Some(candidate);
    output.bytes = 0;
    output.line = 1;
    output.column = 0;
    output.has_content = false;
    source_pass(source, &mut output)?;
    output.check()?;
    if output.bytes != needed {
        return Err(Error::Invalid);
    }
    output.value.ok_or(Error::Invalid)
}
fn source_pass(source: &str, output: &mut Output<'_>) -> Result<(), Error> {
    let mut chars = source.chars().peekable();
    let mut scratch = [0; 8];
    while let Some(ch) = chars.next() {
        output.check()?;
        if ch == '\n' {
            output.newline()?;
        } else {
            output.begin()?;
            output.push(crate::text_projection::scalar(
                ch,
                chars.peek().copied(),
                &mut scratch,
            ))?;
        }
    }
    Ok(())
}
struct Output<'a> {
    value: Option<String>,
    bytes: usize,
    limit: usize,
    geometry: Geometry,
    line: u64,
    column: usize,
    has_content: bool,
    has_semantic: bool,
    cancel: &'a AtomicBool,
    config: Option<&'a PresentationConfig>,
    style: DisplayStyle,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct DisplayStyle {
    colour: Color,
    bits: u32,
}
impl DisplayStyle {
    const PLAIN: Self = Self {
        colour: Color::Reset,
        bits: 0,
    };
}
impl Output<'_> {
    fn role(&self, role: Role, bits: u32) -> DisplayStyle {
        self.config
            .filter(|c| c.coloured)
            .map_or(DisplayStyle::PLAIN, |c| DisplayStyle {
                colour: c.style(role).fg.unwrap_or(Color::Reset),
                bits,
            })
    }
    fn set_style(&mut self, style: DisplayStyle) -> Result<(), Error> {
        if self.style == style {
            return Ok(());
        }
        // All control sequences are owned ASCII. No temporary String or domain copy.
        self.push("\x1b[0")?;
        if style.bits & 1 != 0 {
            self.push(";3")?;
        }
        if style.bits & 2 != 0 {
            self.push(";1")?;
        }
        if style.bits & 4 != 0 {
            self.push(";9")?;
        }
        if style.bits & 16 != 0 {
            self.push(";4")?;
        }
        if let Color::Rgb(r, g, b) = style.colour {
            self.push(";38;2;")?;
            let mut digits = [0; 20];
            self.push(Self::digits(u64::from(r), &mut digits))?;
            self.push(";")?;
            self.push(Self::digits(u64::from(g), &mut digits))?;
            self.push(";")?;
            self.push(Self::digits(u64::from(b), &mut digits))?;
        }
        self.push("m")?;
        self.style = style;
        Ok(())
    }
    fn check(&self) -> Result<(), Error> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
    fn push(&mut self, text: &str) -> Result<(), Error> {
        self.check()?;
        self.bytes = self
            .bytes
            .checked_add(text.len())
            .filter(|n| *n <= self.limit)
            .ok_or(Error::Budget)?;
        if let Some(value) = self.value.as_mut() {
            // The complete counting pass reserved the entire allocation. No growth during copy.
            if self.bytes > value.capacity() {
                return Err(Error::Budget);
            }
            let mut remaining = text;
            while !remaining.is_empty() {
                self.check()?;
                let mut end = remaining.len().min(4096);
                while !remaining.is_char_boundary(end) {
                    end -= 1;
                }
                self.value
                    .as_mut()
                    .ok_or(Error::Invalid)?
                    .push_str(&remaining[..end]);
                remaining = &remaining[end..];
            }
        }
        Ok(())
    }
    fn digits(number: u64, buffer: &mut [u8; 20]) -> &str {
        let mut number = number;
        let mut start = buffer.len();
        loop {
            start -= 1;
            buffer[start] = b'0' + (number % 10) as u8;
            number /= 10;
            if number == 0 {
                break;
            }
        }
        std::str::from_utf8(&buffer[start..]).expect("ASCII digits")
    }
    fn gutter(&self) -> usize {
        if !self.geometry.line_numbers {
            return 0;
        }
        let mut buffer = [0; 20];
        Self::digits(self.line, &mut buffer).len() + 3
    }
    fn begin(&mut self) -> Result<(), Error> {
        if !self.has_content && self.geometry.line_numbers {
            let saved = self.style;
            self.set_style(self.role(Role::Secondary, 0))?;
            let mut buffer = [0; 20];
            self.push(Self::digits(self.line, &mut buffer))?;
            self.push(" | ")?;
            self.set_style(saved)?;
        }
        self.has_content = true;
        Ok(())
    }
    fn newline(&mut self) -> Result<(), Error> {
        // A real empty semantic line has an ordinal; never eagerly create the next gutter.
        self.begin()?;
        self.set_style(DisplayStyle::PLAIN)?;
        self.push("\n")?;
        self.line = self.line.checked_add(1).ok_or(Error::Invalid)?;
        self.column = 0;
        self.has_content = false;
        self.has_semantic = false;
        Ok(())
    }
    fn end_line(&mut self) -> Result<(), Error> {
        if self.has_content {
            self.newline()?;
        }
        Ok(())
    }
    fn before_unit(&mut self, width: usize) -> Result<(), Error> {
        let useful = usize::from(self.geometry.columns).saturating_sub(self.gutter());
        if useful != 0
            && self.column != 0
            && self.column.checked_add(width).ok_or(Error::Invalid)? > useful
        {
            self.newline()?;
        }
        self.begin()?;
        self.column = self.column.checked_add(width).ok_or(Error::Invalid)?;
        Ok(())
    }
    fn literal(&mut self, text: &str) -> Result<(), Error> {
        // Structural decoration is owned ASCII, never a borrowed source/target.
        for ch in text.chars() {
            if ch == '\n' {
                self.newline()?;
            } else {
                self.before_unit(1)?;
                self.set_style(self.role(Role::Secondary, 0))?;
                let mut bytes = [0; 4];
                self.push(ch.encode_utf8(&mut bytes))?;
            }
        }
        Ok(())
    }
}

/// Plain structural output. Optional colour will use the shared role provider, not Source syntax.
pub(crate) fn prepare(
    document: Arc<Document>,
    geometry: Geometry,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, Error> {
    prepare_config(document, geometry, None, cancel, limit)
}
pub(crate) fn prepare_with_config(
    document: Arc<Document>,
    columns: u16,
    config: &PresentationConfig,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, Error> {
    prepare_config(
        document,
        Geometry {
            columns,
            line_numbers: config.line_numbers,
        },
        Some(config),
        cancel,
        limit,
    )
}
fn prepare_config(
    document: Arc<Document>,
    geometry: Geometry,
    config: Option<&PresentationConfig>,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, Error> {
    let budget = ConsumerBudget::new(&document)?;
    let limit = limit.min(MAX_OUTPUT);
    let mut output = Output {
        value: None,
        bytes: 0,
        limit,
        geometry,
        line: 1,
        column: 0,
        has_content: false,
        has_semantic: false,
        cancel,
        config,
        style: DisplayStyle::PLAIN,
    };
    walk(&document, &budget, 0, &mut output)?;
    output.set_style(DisplayStyle::PLAIN)?;
    output.check()?;
    let needed = output.bytes;
    let mut value = String::new();
    value.try_reserve_exact(needed).map_err(|_| Error::Budget)?;
    if value.capacity() > limit {
        return Err(Error::Budget);
    }
    output.value = Some(value);
    output.bytes = 0;
    output.line = 1;
    output.column = 0;
    output.has_content = false;
    output.has_semantic = false;
    output.style = DisplayStyle::PLAIN;
    walk(&document, &budget, 0, &mut output)?;
    output.set_style(DisplayStyle::PLAIN)?;
    output.check()?;
    if output.bytes != needed {
        return Err(Error::Invalid);
    }
    output.value.ok_or(Error::Invalid)
}

fn domain(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    output: &mut Output<'_>,
) -> Result<(), Error> {
    let record = document.domains().get(id as usize).ok_or(Error::Invalid)?;
    domain_range(document, budget, id, 0, record.run_count() as usize, output)
}
fn domain_range(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    mut start: usize,
    last: usize,
    output: &mut Output<'_>,
) -> Result<(), Error> {
    let record = document.domains().get(id as usize).ok_or(Error::Invalid)?;
    if start > last || last > record.run_count() as usize {
        return Err(Error::Invalid);
    }
    let original_start = start;
    while start < last {
        output.check()?;
        let run = &document.runs()[record.first_run() as usize + start];
        if run.kind() == 5 {
            resource(document, budget, run.resource(), output)?;
            output.has_semantic = true;
            start += 1;
            continue;
        }
        let mut end = start + 1;
        while end < last && document.runs()[record.first_run() as usize + end].kind() != 5 {
            output.check()?;
            end += 1;
        }
        let text = SegmentedText::prepare(document.clone(), id, start..end, budget, output.cancel)?;
        let index = SegmentedIndex::prepare(text, budget, output.cancel)?;
        let mut point = index.start();
        let mut work = DocumentWork::preparation(output.cancel);
        while let Some(unit) = index.step(&mut point, 4096, &mut work)? {
            if unit.newline {
                output.newline()?;
                continue;
            }
            let mut tab = false;
            if unit.projected.len() == 1 {
                index
                    .text()
                    .visit_projected(unit.projected.clone(), &mut work, |s| {
                        tab = s == "\t";
                        Ok(())
                    })?;
            }
            if tab {
                let width = 8 - output.column % 8;
                let useful = usize::from(output.geometry.columns).saturating_sub(output.gutter());
                if useful != 0
                    && output.column != 0
                    && output.column.checked_add(width).ok_or(Error::Invalid)? > useful
                {
                    output.newline()?;
                }
            }
            let width = if tab {
                8 - output.column % 8
            } else {
                unit.width
            };
            output.before_unit(width)?;
            output.has_semantic = true;
            let mut bits = 0;
            let mut linked = false;
            index
                .text()
                .visit_coverage(unit.projected.clone(), &mut work, |coverage| {
                    bits |= coverage.styles;
                    linked |= coverage.link.is_some();
                    Ok(())
                })?;
            let heading = record.kind() == 2
                || (record.kind() == 0 && document.blocks()[record.owner() as usize].kind() == 2);
            let role = if heading {
                Role::Heading
            } else if bits & 8 != 0 || linked {
                Role::Accent
            } else {
                Role::Value
            };
            output.set_style(output.role(
                role,
                bits | if heading { 2 } else { 0 } | if linked { 16 } else { 0 },
            ))?;
            if tab {
                for _ in 0..width {
                    output.push(" ")?;
                }
            } else {
                // A giant cluster remains complete; SYMBOL_BYTES is only a TUI cell limit.
                index
                    .text()
                    .visit_projected(unit.projected, &mut work, |s| output.push(s))?;
            }
        }
        start = end;
    }
    // Inline output retains destinations using safe labels; no destination is opened.
    let mut previous = NONE;
    for run in &document.runs()
        [record.first_run() as usize + original_start..record.first_run() as usize + last]
    {
        output.check()?;
        if run.link() != NONE && run.link() != previous {
            let link = document
                .links()
                .get(run.link() as usize)
                .ok_or(Error::Invalid)?;
            output.end_line()?;
            output.literal("[link: ")?;
            safe_reference(document, budget, link.target_decoded(), output)?;
            output.literal("]")?;
            output.end_line()?;
        }
        previous = run.link();
    }
    Ok(())
}

fn safe_reference(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    output: &mut Output<'_>,
) -> Result<(), Error> {
    if id == NONE {
        return Ok(());
    }
    let index = TextRefIndex::prepare(
        document.clone(),
        id,
        budget,
        output.cancel,
        IndexProfile::DEFAULT,
    )?;
    let mut at = index.start();
    let mut work = DocumentWork::preparation(output.cancel);
    while let Some(unit) = index.step(&mut at, 4096, &mut work)? {
        if unit.newline {
            output.newline()?;
            continue;
        }
        let mut tab = false;
        if unit.projected.len() == 1 {
            index.visit_projected(unit.projected.clone(), &mut work, |s| {
                tab = s == "\t";
                Ok(())
            })?;
        }
        if tab {
            let width = 8 - output.column % 8;
            let useful = usize::from(output.geometry.columns).saturating_sub(output.gutter());
            if useful != 0
                && output.column != 0
                && output.column.checked_add(width).ok_or(Error::Invalid)? > useful
            {
                output.newline()?;
            }
        }
        let width = if tab {
            8 - output.column % 8
        } else {
            unit.width
        };
        output.before_unit(width)?;
        output.has_semantic = true;
        output.set_style(output.role(Role::Secondary, 0))?;
        if tab {
            for _ in 0..width {
                output.push(" ")?;
            }
        } else {
            index.visit_projected(unit.projected, &mut work, |s| output.push(s))?;
        }
    }
    Ok(())
}

fn resource(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    output: &mut Output<'_>,
) -> Result<(), Error> {
    use crate::document_roles::{ResourceKind, resource_descriptor};
    let mut work = DocumentWork::preparation(output.cancel);
    let resource = resource_descriptor(document, id, &mut work)?;
    output.literal(match resource.kind() {
        ResourceKind::Image => "[Image pending: ",
        ResourceKind::Mermaid => "[Mermaid pending: ",
        ResourceKind::MathInline | ResourceKind::MathBlock => "[Math pending: ",
        ResourceKind::WikiEmbed => "[Wiki embed pending: ",
    })?;
    safe_reference(document, budget, resource.input_ref(), output)?;
    if let Some(alt) = resource.alt_domain() {
        output.literal("; alt: ")?;
        domain(document, budget, alt, output)?;
    }
    if let Some(title) = resource.title_ref() {
        output.literal("; title: ")?;
        safe_reference(document, budget, title, output)?;
    }
    output.literal("]")
}

fn enter(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    ordinal: u32,
    output: &mut Output<'_>,
) -> Result<(), Error> {
    output.check()?;
    let block = document.blocks().get(id as usize).ok_or(Error::Invalid)?;
    use crate::document_roles::{BlockRole, TaskState, block_role};
    let mut work = DocumentWork::preparation(output.cancel);
    match block_role(document, id, &mut work)? {
        BlockRole::Heading { level } => {
            for _ in 0..level {
                output.literal("#")?;
            }
            output.literal(" ")?;
        }
        BlockRole::Quote => output.literal("> ")?,
        BlockRole::Callout {
            kind_text,
            title_domain,
            ..
        } => {
            output.literal("[callout ")?;
            safe_reference(document, budget, kind_text, output)?;
            output.literal("] ")?;
            if let Some(title) = title_domain {
                domain(document, budget, title, output)?;
            }
            output.end_line()?;
        }
        BlockRole::Item { task } => {
            match block_role(document, block.parent(), &mut work)? {
                BlockRole::List {
                    ordered: true,
                    start,
                    ..
                } => {
                    let n = u64::from(start)
                        .checked_add(u64::from(ordinal))
                        .ok_or(Error::Invalid)?;
                    let mut bytes = [0; 20];
                    output.literal(Output::digits(n, &mut bytes))?;
                    output.literal(". ")?;
                }
                BlockRole::List { ordered: false, .. } => output.literal("- ")?,
                _ => return Err(Error::Invalid),
            }
            match task {
                TaskState::Unchecked => output.literal("[ ] ")?,
                TaskState::Checked => output.literal("[x] ")?,
                TaskState::Absent => (),
            }
        }
        BlockRole::Cell { .. } => {
            let mut bytes = [0; 20];
            output.literal("column ")?;
            output.literal(Output::digits(u64::from(ordinal) + 1, &mut bytes))?;
            output.literal(": ")?;
        }
        BlockRole::Code {
            mermaid: Some(_), ..
        } => output.literal("[Mermaid pending]\n")?,
        BlockRole::Rule => output.literal("---\n")?,
        BlockRole::Footnote { label } => {
            output.literal("[^")?;
            safe_reference(document, budget, label, output)?;
            output.literal("]: ")?;
        }
        _ => (),
    }

    Ok(())
}
#[derive(Clone, Copy)]
struct Frame {
    block: u32,
    ordinal: u32,
    next: usize,
    entered: bool,
}
fn walk(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    output: &mut Output<'_>,
) -> Result<(), Error> {
    // Exactly the validated maximum depth plus root. Actual vector capacity is charged.
    let mut stack = ChargedVec::new(budget, 65)?;
    stack.push(Frame {
        block: id,
        ordinal: 0,
        next: 0,
        entered: false,
    })?;
    while let Some(mut frame) = stack.pop() {
        output.check()?;
        if !frame.entered {
            enter(document, budget, frame.block, frame.ordinal, output)?;
            frame.entered = true;
        }
        let block = document
            .blocks()
            .get(frame.block as usize)
            .ok_or(Error::Invalid)?;
        // Items can interleave child blocks and runs. Never move all domain text before children.
        let items = &document.items()
            [block.first_item() as usize..(block.first_item() + block.item_count()) as usize];
        if frame.next == items.len() {
            if frame.block != 0 {
                output.end_line()?;
            }
            continue;
        }
        let mut at = frame.next;
        output.check()?;
        let item = items[at];
        if item.kind() == 0 {
            // A child block is a semantic boundary. A parent marker can share its
            // first child's line, but preceding body text must finish before it.
            if output.has_semantic {
                output.end_line()?;
            }
            frame.next = at + 1;
            stack.push(frame)?;
            stack.push(Frame {
                block: item.target(),
                ordinal: item.ordinal(),
                next: 0,
                entered: false,
            })?;
        } else {
            let first = document.runs()[item.target() as usize].ordinal() as usize;
            let mut last = first + 1;
            at += 1;
            while at < items.len() && items[at].kind() == 1 {
                output.check()?;
                let run = document
                    .runs()
                    .get(items[at].target() as usize)
                    .ok_or(Error::Invalid)?;
                if run.ordinal() as usize != last {
                    return Err(Error::Invalid);
                }
                last += 1;
                at += 1;
            }
            domain_range(document, budget, block.domain(), first, last, output)?;
            frame.next = at;
            stack.push(frame)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        document_budget::ActorArena, document_model::Profile, document_obsidian::Control,
        document_source::DocumentSource, input::SourceOrigin,
    };
    use std::time::{Duration, Instant};
    fn config(coloured: bool, numbers: bool) -> PresentationConfig {
        use clap::Parser;
        use std::sync::atomic::AtomicU64;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "r3viewer-document-inline-{}-{}.toml",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path,"[colours]\nvalue='default'\nheading='#0d0e0f'\naccent='#010203'\nsecondary='#040506'\n").unwrap();
        let cli = crate::cli::Cli::parse_from([
            "viewer",
            "--config",
            path.to_str().unwrap(),
            "--colour",
            if coloured { "always" } else { "never" },
            if numbers {
                "--line-numbers"
            } else {
                "--no-line-numbers"
            },
        ]);
        let result =
            PresentationConfig::load(&cli, true, Arc::new(AtomicBool::new(false))).unwrap();
        std::fs::remove_file(path).unwrap();
        result
    }
    fn document(source: &str) -> Arc<Document> {
        let cancel = AtomicBool::new(false);
        let control = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(source.to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        let candidate =
            crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &control)
                .unwrap();
        let depth = candidate.max_depth;
        Arc::new(Document::from_validated(
            crate::document_validate::quarantine(candidate, depth, &control).unwrap(),
        ))
    }
    fn output(source: &str, columns: u16, numbers: bool) -> String {
        prepare(
            document(source),
            Geometry {
                columns,
                line_numbers: numbers,
            },
            &AtomicBool::new(false),
            MAX_OUTPUT,
        )
        .unwrap()
    }
    #[test]
    fn complete_plain_and_heading() {
        assert_eq!(output("hello", 80, false), "hello\n");
        assert_eq!(output("# Title", 80, false), "# Title\n");
        assert_eq!(output("", 80, false), "");
    }
    #[test]
    fn cluster_crosses_style_runs_and_wraps_whole() {
        assert_eq!(output("a**\u{301}**", 1, false), "a\u{301}\n");
        assert_eq!(output("abcd", 2, false), "ab\ncd\n");
        assert_eq!(output("abcd", 0, false), "abcd\n");
    }
    #[test]
    fn table_vertical_order() {
        assert_eq!(
            output("|A|B|\n|-|-|\n|x|y|", 80, false),
            "column 1: A\ncolumn 2: B\ncolumn 1: x\ncolumn 2: y\n"
        );
    }
    #[test]
    fn output_budget_and_cancel_are_transactional() {
        let doc = document("hello");
        let geometry = Geometry {
            columns: 80,
            line_numbers: false,
        };
        assert_eq!(
            prepare(doc.clone(), geometry, &AtomicBool::new(false), 6).unwrap(),
            "hello\n"
        );
        assert_eq!(
            prepare(doc.clone(), geometry, &AtomicBool::new(false), 5),
            Err(Error::Budget)
        );
        assert_eq!(
            prepare(doc, geometry, &AtomicBool::new(true), 6),
            Err(Error::Cancelled)
        );
    }
    #[test]
    fn numbering_does_not_create_final_empty_line() {
        assert_eq!(output("one\n\ntwo", 80, true), "1 | one\n2 | two\n");
        let source = (0..10).map(|_| "x\n").collect::<String>();
        // Fenced content preserves semantic hard breaks, unlike paragraph soft breaks.
        let source = "```\n".to_owned() + &source + "```";
        let expected = (1..=10).map(|n| format!("{n} | x\n")).collect::<String>();
        assert_eq!(output(&source, 80, true), expected);
    }
    #[test]
    fn giant_cluster_is_complete_and_controls_safe() {
        let source = "BEFORE a".to_owned() + &"\u{301}".repeat(9000) + " AFTER";
        assert_eq!(output(&source, 0, false), source + "\n");
        assert_eq!(output("A\u{1b}B", 80, false), "A␛B\n");
    }
    #[test]
    fn nested_block_stays_between_item_text() {
        let result = output("- before\n  - inside\n\n  after", 80, false);
        assert!(result.find("before").unwrap() < result.find("inside").unwrap());
        assert!(result.find("inside").unwrap() < result.find("after").unwrap());
    }
    #[test]
    fn tab_recomputes_after_wrap_and_empty_lines_have_gutters() {
        assert_eq!(output("```\na\tb\n```", 4, false), "a\n        \nb\n");
        assert_eq!(output("```\na\n\nb\n```", 80, true), "1 | a\n2 | \n3 | b\n");
    }
    #[test]
    fn inline_links_retain_safe_destination_without_io() {
        assert_eq!(
            output("read [site](https://example.test)", 80, false),
            "read site\n[link: https://example.test]\n"
        );
    }
    #[test]
    fn tight_list_code_child_keeps_block_boundaries() {
        assert_eq!(
            output("- before\n  ```text\n  code\n  ```\n  after\n", 80, false),
            "- before\ncode\nafter\n"
        );
        assert_eq!(
            output("- first\n\n- second", 80, false),
            "- first\n- second\n"
        );
    }
    #[test]
    fn coloured_cluster_uses_union_styles_without_sgr_inside() {
        let doc = document("a**\u{301}**");
        let config = config(true, false);
        let expected = "\x1b[0;1ma\u{301}\x1b[0m\n";
        assert_eq!(
            prepare_with_config(
                doc.clone(),
                1,
                &config,
                &AtomicBool::new(false),
                expected.len()
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            prepare_with_config(doc, 1, &config, &AtomicBool::new(false), expected.len() - 1),
            Err(Error::Budget)
        );
    }
    #[test]
    fn never_colour_and_configured_gutter_remain_plain() {
        let config = config(false, true);
        assert_eq!(
            prepare_with_config(
                document("# Title"),
                80,
                &config,
                &AtomicBool::new(false),
                MAX_OUTPUT
            )
            .unwrap(),
            "1 | # Title\n"
        );
    }
    #[test]
    fn document_failure_uses_original_complete_source_without_index() {
        let config = config(false, false);
        let result = render(document("[x](y)"), 80, &config, &AtomicBool::new(false), 6).unwrap();
        assert_eq!(result.mode, Mode::Source);
        assert_eq!(result.content, "[x](y)");
        assert!(result.diagnostic.unwrap().contains("Fuente completa"));
        assert!(matches!(
            render(document("[x](y)"), 80, &config, &AtomicBool::new(false), 5),
            Err(Error::Budget)
        ));
        assert!(matches!(
            render(document("[x](y)"), 80, &config, &AtomicBool::new(true), 6),
            Err(Error::Cancelled)
        ));
    }
    #[test]
    fn source_fallback_numbers_crlf_and_has_no_tui_line_limit() {
        assert_eq!(
            source_fallback("A\r\n\r\nB\u{1b}", true, &AtomicBool::new(false), 100).unwrap(),
            "1 | A\r\n2 | \r\n3 | B␛"
        );
        assert_eq!(
            source_fallback("", true, &AtomicBool::new(false), 0).unwrap(),
            ""
        );
        assert_eq!(
            source_fallback("x\n", true, &AtomicBool::new(false), 6).unwrap(),
            "1 | x\n"
        );
        assert_eq!(
            source_fallback("x\n", true, &AtomicBool::new(false), 5),
            Err(Error::Budget)
        );
        let source = "\n".repeat(1_000_001);
        let complete =
            source_fallback(&source, false, &AtomicBool::new(false), source.len()).unwrap();
        assert_eq!(complete, source);
    }
    #[test]
    fn resource_title_decoded_controls_and_typed_image() {
        let source = "![ALT](x \"A\u{1b}\u{85}&amp;Z\")";
        assert_eq!(
            output(source, 0, false),
            "[Image pending: x; alt: ALT; title: A␛\\u{0085}&Z]\n"
        );
        assert_eq!(
            output("![[note]]", 0, false),
            "[Wiki embed pending: note; alt: note]\n"
        );
    }
    #[test]
    fn resource_title_giant_cluster_stays_complete_at_narrow_width() {
        let cluster = "a".to_owned() + &"\u{301}".repeat(9000);
        let title = cluster.clone() + " AFTER";
        let source = format!("![ALT](x \"{title}\")");
        let expected = format!("[Image pending: x; alt: ALT; title: {title}]\n");
        assert_eq!(output(&source, 0, false), expected);
        let narrow = output(&source, 1, false);
        assert!(
            narrow.contains(&cluster),
            "whole cluster must be contiguous despite width and TUI symbol cap"
        );
        assert!(
            narrow.contains("A\nF\nT\nE\nR"),
            "tail must remain after giant cluster"
        );
        let doc = document(&source);
        assert_eq!(
            prepare(
                doc,
                Geometry {
                    columns: 0,
                    line_numbers: false
                },
                &AtomicBool::new(false),
                expected.len() - 1
            ),
            Err(Error::Budget)
        );
    }
}
