use super::*;
use crate::{
    document_budget::ActorArena,
    document_layout::Geometry,
    document_model::{Document, Profile},
    document_terminal::{BudgetedTerminal, RenderError},
    document_view::{DocumentPreparationAuthority, DocumentView, start_document},
    preparation_owner::{Outcome, PreparedCompletion},
};
use clap::Parser;
use ratatui::{
    backend::{Backend, ClearType, CrosstermBackend, WindowSize},
    buffer::Cell,
    layout::{Position, Size},
};
use std::{
    io::{self, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const CAPTURE_BYTES: usize = 65536;
#[derive(Clone)]
struct Capture(Arc<Mutex<Vec<u8>>>, Arc<AtomicBool>);
impl Capture {
    fn new() -> Self {
        let mut v = Vec::new();
        v.try_reserve_exact(CAPTURE_BYTES).unwrap();
        assert_eq!(v.capacity(), CAPTURE_BYTES);
        Self(Arc::new(Mutex::new(v)), Arc::new(AtomicBool::new(false)))
    }
    fn bytes(&self) -> Vec<u8> {
        self.0.lock().unwrap().clone()
    }
    fn clear(&self) {
        self.0.lock().unwrap().clear()
    }
}
impl Write for Capture {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        let mut v = self.0.lock().unwrap();
        if v.len()
            .checked_add(b.len())
            .is_none_or(|n| n > CAPTURE_BYTES)
        {
            return Err(io::Error::other("bounded harness capture"));
        }
        v.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.1.load(Ordering::Acquire) {
            Err(io::Error::other("controlled harness flush error"))
        } else {
            Ok(())
        }
    }
}
struct NativeWriterBackend {
    native: CrosstermBackend<Capture>,
    size: Size,
}
impl Backend for NativeWriterBackend {
    type Error = io::Error;
    fn draw<'a, I>(&mut self, c: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.native.draw(c)
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        self.native.hide_cursor()
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        self.native.show_cursor()
    }
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::new(0, 0))
    }
    fn set_cursor_position<P: Into<Position>>(&mut self, p: P) -> io::Result<()> {
        self.native.set_cursor_position(p)
    }
    fn clear(&mut self) -> io::Result<()> {
        self.native.clear()
    }
    fn clear_region(&mut self, t: ClearType) -> io::Result<()> {
        self.native.clear_region(t)
    }
    fn size(&self) -> io::Result<Size> {
        Ok(self.size)
    }
    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size,
            pixels: Size::new(0, 0),
        })
    }
    fn flush(&mut self) -> io::Result<()> {
        Backend::flush(&mut self.native)
    }
}
fn geometry(w: u16, h: u16) -> Geometry {
    Geometry {
        x: 0,
        y: 0,
        width: w,
        height: h,
        left: 0,
    }
}
fn view(raw: &str, g: Geometry) -> DocumentView {
    let cancel = AtomicBool::new(false);
    let ctl = crate::document_obsidian::Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let original = Arc::new(raw.to_owned());
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            original.clone(),
            crate::input::SourceOrigin::Literal,
            7,
            9,
            &cancel,
        )
        .unwrap(),
    );
    let c = crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
        .unwrap();
    let depth = crate::document_validate::validate(&c, &ctl).unwrap();
    let validated = crate::document_validate::quarantine(c, depth, &ctl).unwrap();
    let doc = Arc::new(Document::from_validated(validated));
    assert!(Arc::ptr_eq(&doc.source().original_handle(), &original));
    let input = DocumentView::initial_input(doc, g).unwrap();
    let binding = input.key().binding;
    let mut authority = DocumentPreparationAuthority::default();
    start_document(input, &mut authority).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !authority.is_finished() && Instant::now() < deadline {
        thread::yield_now()
    }
    assert!(authority.is_finished());
    let Some(PreparedCompletion::Document(Outcome::Ready(prepared), receipt)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("real factory completion")
    };
    assert!(receipt.joined);
    DocumentView::from_prepared(prepared).unwrap()
}
fn config(coloured: bool) -> PresentationConfig {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut cli = crate::cli::Cli::parse_from(["r3viewer", "--value", "fixture"]);
    cli.colour = Some(if coloured {
        crate::config::Colour::Always
    } else {
        crate::config::Colour::Never
    });
    PresentationConfig::load(&cli, true, cancel).unwrap()
}
fn owner(w: u16, h: u16) -> (BudgetedTerminal<NativeWriterBackend>, Capture) {
    let capture = Capture::new();
    let backend = NativeWriterBackend {
        native: CrosstermBackend::new(capture.clone()),
        size: Size::new(w, h),
    };
    let mut owner = BudgetedTerminal::new(backend).unwrap();
    owner
        .prepare_area(Rect::new(0, 0, w, h), &AtomicBool::new(false), || true)
        .unwrap();
    capture.clear();
    (owner, capture)
}
#[test]
fn dense_factory_real_native_writer_gate() {
    let raw = format!("{}  \n", "X".repeat(120)).repeat(40);
    assert_eq!(raw.len(), 4920);
    for (w, h, glyphs) in [(80, 24, 1440), (120, 40, 4800)] {
        let v = view(&raw, geometry(w, h));
        let text = v.layout().text().expect("dense Text");
        assert_eq!(text.glyphs().len(), glyphs);
        assert_eq!(text.rows().len(), usize::from(h));
        assert_eq!(text.symbols(), "X".repeat(glyphs));
        let (mut owner, bytes) = owner(w, h);
        let c = config(false);
        let result =
            owner.draw_document(&v, &c, &AtomicBool::new(false), |b| b == v.render_binding());
        eprintln!(
            "RENDERER_DENSE {w}x{h} window={} layout_work={} result={result:?}",
            text.bytes(),
            text.work
        );
        let receipt = result.expect("same Work must admit dense renderer");
        assert!(receipt.total <= DOCUMENT_FRAME_WORK);
        assert_eq!(bytes.bytes().iter().filter(|&&b| b == b'X').count(), glyphs);
        bytes.clear();
        owner
            .draw_document(&v, &c, &AtomicBool::new(false), |b| b == v.render_binding())
            .unwrap();
        assert!(!bytes.bytes().contains(&b'X'));
    }
}
#[test]
fn native_preclears_and_stale_cancel() {
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let a = view("ab", geometry(3, 1));
    let wide = view("♥️", geometry(3, 1));
    let narrow = view("x", geometry(3, 1));
    let (mut t, bytes) = owner(3, 1);
    t.draw_document(&a, &c, &cancel, |_| true).unwrap();
    bytes.clear();
    t.draw_document(&wide, &c, &cancel, |_| true).unwrap();
    let written = String::from_utf8(bytes.bytes()).unwrap();
    assert!(
        written.contains("\u{1b}[1;2H \u{1b}[1;1H♥️"),
        "preclear before base: {written:?}"
    );
    bytes.clear();
    t.draw_document(&narrow, &c, &cancel, |_| true).unwrap();
    assert!(String::from_utf8(bytes.bytes()).unwrap().contains('x'));
    bytes.clear();
    assert!(matches!(
        t.draw_document(&wide, &c, &cancel, |_| false),
        Err(RenderError::Owner(crate::document_terminal::Error::Stale))
    ));
    assert!(bytes.bytes().is_empty());
    let mut calls = 0;
    assert!(matches!(
        t.draw_document(&wide, &c, &cancel, |_| {
            calls += 1;
            if calls == 3 {
                cancel.store(true, Ordering::Release)
            }
            true
        }),
        Err(RenderError::Owner(
            crate::document_terminal::Error::Cancelled
        ))
    ));
    assert!(bytes.bytes().is_empty());
}
#[test]
fn semantic_span_style_matches_inline() {
    let v = view(
        "a**\u{301}** *i* ~~s~~ \x60c\x60 [l](target)",
        geometry(30, 2),
    );
    let c = config(true);
    let p = config(false);
    let holder = v.layout().text().unwrap().glyphs();
    let first = holder.get(0).unwrap();
    assert_eq!(first.styles & 2, 2);
    assert!(style(first, &c).add_modifier.contains(Modifier::BOLD));
    assert_eq!(style(first, &p), Style::default());
    assert!(
        holder
            .iter()
            .any(|g| style(g, &c).add_modifier.contains(Modifier::ITALIC))
    );
    assert!(
        holder
            .iter()
            .any(|g| style(g, &c).add_modifier.contains(Modifier::CROSSED_OUT))
    );
    assert!(
        holder
            .iter()
            .any(|g| style(g, &c).add_modifier.contains(Modifier::UNDERLINED))
    );
}

#[test]
fn heap_copy_is_charged_and_drop_releases_actual_symbols() {
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let raw = format!("a{}", "\u{301}".repeat(16));
    assert_eq!(raw.len(), 33);
    let v = view(&raw, geometry(8, 2));
    let baseline = v.budget().used();
    let (mut t, bytes) = owner(8, 2);
    t.draw_document(&v, &c, &cancel, |b| b == v.render_binding())
        .unwrap();
    assert!(t.receipt().unwrap().symbol_heap.iter().sum::<usize>() >= 33);
    assert_eq!(v.budget().used(), baseline + 33);
    assert!(String::from_utf8(bytes.bytes()).unwrap().contains(&raw));
    drop(t);
    assert_eq!(v.budget().used(), baseline);
}
#[test]
fn foreign_geometry_and_cancelled_input_do_not_write() {
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let v = view("text", geometry(9, 2));
    let (mut t, bytes) = owner(8, 2);
    assert!(matches!(
        t.draw_document(&v, &c, &cancel, |_| true),
        Err(RenderError::Admission(Error::Invalid))
    ));
    assert!(bytes.bytes().is_empty());
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        t.draw_document(&v, &c, &cancel, |_| true),
        Err(RenderError::Owner(
            crate::document_terminal::Error::Cancelled
        ))
    ));
    assert!(bytes.bytes().is_empty());
}
#[test]
fn shared_work_exact_guard() {
    let cancel = AtomicBool::new(false);
    let mut w = DocumentWork::bounded(&cancel, DOCUMENT_FRAME_WORK).unwrap();
    w.admit_structure(DOCUMENT_FRAME_WORK).unwrap();
    assert!(w.admit_structure(1).is_err());
    assert!(DocumentWork::bounded(&cancel, DOCUMENT_FRAME_WORK + 1).is_err());
    cancel.store(true, Ordering::Release);
    assert!(
        DocumentWork::bounded(&cancel, DOCUMENT_FRAME_WORK)
            .unwrap()
            .admit_structure(0)
            .is_err()
    );
}

#[test]
fn measured_old4m_exact_total_and_previous_receipt_are_closed() {
    let raw = format!("{}  \n", "X".repeat(120)).repeat(40);
    let v = view(&raw, geometry(120, 40));
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let (mut t, bytes) = owner(120, 40);
    let previous = t.previous_frame();
    let e = prepare_limit(
        &v,
        &c,
        Rect::new(0, 0, 120, 40),
        &previous,
        &cancel,
        4194304,
    )
    .err()
    .unwrap();
    let Error::Work {
        required,
        limit,
        parts,
    } = e
    else {
        panic!("old4m Work")
    };
    assert_eq!((required, limit), (7063279, 4194304));
    assert_eq!(
        parts,
        [1327023, 710400, 715200, 921600, 465600, 460800, 2462656]
    );
    assert!(bytes.bytes().is_empty());
    assert!(
        prepare_limit(
            &v,
            &c,
            Rect::new(0, 0, 120, 40),
            &previous,
            &cancel,
            required - 1
        )
        .is_err()
    );
    let (plan, permit) = prepare_limit(
        &v,
        &c,
        Rect::new(0, 0, 120, 40),
        &previous,
        &cancel,
        required,
    )
    .unwrap();
    assert!(permit.matches(&plan));
    drop((plan, permit));
    t.draw_document(&v, &c, &cancel, |b| b == v.render_binding())
        .unwrap();
    bytes.clear();
    let previous = t.previous_frame();
    assert_eq!(previous.window(), 122848);
    assert_eq!(previous.payload(), 4800);
    let (plan, _) = prepare(&v, &c, Rect::new(0, 0, 120, 40), &previous, &cancel).unwrap();
    assert!(plan.frame_bytes() > 245696 + 9600);
    assert_eq!(plan.work().diff, 470400);
    let other = view("other", geometry(120, 40));
    let (otherplan, otherpermit) =
        prepare(&other, &c, Rect::new(0, 0, 120, 40), &previous, &cancel).unwrap();
    assert!(!otherpermit.matches(&plan));
    drop((plan, otherplan, otherpermit));
}
#[test]
fn packed_roundtrip_and_class_overflow_preserve_fields() {
    let v = view("packing", geometry(20, 2));
    crate::document_layout::check_storage_roundtrip(v.budget());
}
#[test]
fn real_factory_resize_and_two_previous_frames() {
    let raw = format!("{}  \n", "X".repeat(120)).repeat(40);
    let mut v = view(&raw, geometry(120, 40));
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let (mut t, bytes) = owner(120, 40);
    t.draw_document(&v, &c, &cancel, |b| b == v.render_binding())
        .unwrap();
    for (w, h, expected) in [(121, 40, 4800), (80, 24, 1440), (120, 40, 4800)] {
        let g = geometry(w, h);
        let i = v
            .request(crate::document_view::Operation::Resize, g)
            .unwrap();
        let b = i.key().binding;
        let mut a = DocumentPreparationAuthority::default();
        start_document(i, &mut a).unwrap();
        let until = Instant::now() + Duration::from_secs(5);
        while !a.is_finished() && Instant::now() < until {
            thread::yield_now()
        }
        assert!(a.is_finished());
        let Some(PreparedCompletion::Document(Outcome::Ready(p), r)) =
            a.take_validated(b, false).unwrap()
        else {
            panic!("resize real")
        };
        assert!(r.joined);
        v.commit(p).unwrap();
        t.prepare_area(Rect::new(0, 0, w, h), &cancel, || true)
            .unwrap();
        bytes.clear();
        let receipt = t
            .draw_document(&v, &c, &cancel, |token| token == v.render_binding())
            .unwrap();
        assert!(receipt.total <= DOCUMENT_FRAME_WORK);
        assert_eq!(v.layout().text().unwrap().glyphs().len(), expected);
        assert_eq!(
            bytes.bytes().iter().filter(|&&b| b == b'X').count(),
            expected
        );
    }
}

#[test]
fn final_cancel_retains_visible_mirror_and_previous_heap_authority() {
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let a = view("ab", geometry(3, 1));
    let wide = view("♥️", geometry(3, 1));
    let narrow = view("x", geometry(3, 1));
    let (mut t, bytes) = owner(3, 1);
    t.draw_document(&a, &c, &cancel, |_| true).unwrap();
    bytes.clear();
    let before = t.receipt().unwrap();
    t.prepare_area(Rect::new(0, 0, 3, 1), &cancel, || true)
        .unwrap();
    assert!(bytes.bytes().is_empty());
    assert_eq!(t.previous_frame().payload(), 2);
    assert_eq!(t.receipt().unwrap(), before);
    let mut calls = 0;
    assert!(matches!(
        t.draw_document(&wide, &c, &cancel, |_| {
            calls += 1;
            if calls == 3 {
                cancel.store(true, Ordering::Release)
            }
            true
        }),
        Err(RenderError::Owner(
            crate::document_terminal::Error::Cancelled
        ))
    ));
    assert!(bytes.bytes().is_empty());
    assert_eq!(t.previous_frame().payload(), 2);
    cancel.store(false, Ordering::Release);
    t.draw_document(&narrow, &c, &cancel, |_| true).unwrap();
    let output = String::from_utf8(bytes.bytes()).unwrap();
    assert!(
        output.contains("x "),
        "recovery erases previous b: {output:?}"
    );

    let raw = format!("a{}", "\u{301}".repeat(16));
    let heap = view(&raw, geometry(3, 1));
    let baseline = heap.budget().used();
    t.draw_document(&heap, &c, &cancel, |_| true).unwrap();
    assert_eq!(heap.budget().used(), baseline + 33);
    let oldheap = t.receipt().unwrap().symbol_heap.iter().sum::<usize>();
    bytes.clear();
    t.prepare_area(Rect::new(0, 0, 3, 1), &cancel, || true)
        .unwrap();
    assert!(bytes.bytes().is_empty());
    assert_eq!(
        t.receipt().unwrap().symbol_heap.iter().sum::<usize>(),
        oldheap
    );
    calls = 0;
    assert!(
        t.draw_document(&wide, &c, &cancel, |_| {
            calls += 1;
            if calls == 3 {
                cancel.store(true, Ordering::Release)
            }
            true
        })
        .is_err()
    );
    assert!(bytes.bytes().is_empty());
    assert_eq!(
        t.receipt().unwrap().symbol_heap.iter().sum::<usize>(),
        oldheap
    );
    assert_eq!(heap.budget().used(), baseline + 33);
    drop(t);
    assert_eq!(heap.budget().used(), baseline);
}

#[test]
fn partial_native_flush_error_poison_is_not_false_atomic_rollback() {
    let c = config(false);
    let cancel = AtomicBool::new(false);
    let raw = format!("a{}", "\u{301}".repeat(16));
    let v = view(&raw, geometry(3, 1));
    let baseline = v.budget().used();
    let (mut t, bytes) = owner(3, 1);
    bytes.1.store(true, Ordering::Release);
    assert!(matches!(
        t.draw_document(&v, &c, &cancel, |_| true),
        Err(RenderError::Owner(
            crate::document_terminal::Error::Backend(_)
        ))
    ));
    assert!(!bytes.bytes().is_empty());
    assert_eq!(v.budget().used(), baseline);
    assert_eq!(t.receipt().unwrap().symbol_heap, [0, 0]);
    bytes.clear();
    bytes.1.store(false, Ordering::Release);
    assert!(matches!(
        t.draw_document(&v, &c, &cancel, |_| true),
        Err(RenderError::Owner(
            crate::document_terminal::Error::Poisoned
        ))
    ));
    assert!(bytes.bytes().is_empty());
}
