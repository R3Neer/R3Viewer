//! Manual Ratatui session over the existing exact-state terminal guard.
use crate::{
    config::{PresentationConfig, Role},
    data::{Cell, DataRef, FileNavigator, FilePart, InspectionRef, Navigator, PathMember},
    inspect_text::{file_leaf_text, primitive_text},
    presentation::{SafeTextModel, cell_width, slice_cells},
    search::{
        Direction, FileSearchHit, QUERY_LIMIT, SearchHit, SearchOutcome, SearchWorker, find_data,
        find_structure, find_visible,
    },
    session::{TerminalGuard, control_terminal},
    structure::{
        Mode as StructureMode, StructureClient, StructureEvent, StructureFailure, StructureShutdown,
    },
    syntax::{
        FallbackReason, RoleSpan, SyntaxClient, SyntaxEvent, SyntaxLimits, SyntaxRole,
        SyntaxShutdown, Viewport,
    },
    syntax_selection::{StructureSelection, SyntaxSelection, fallback_diagnostic},
    worker_process::{ReapState, ReapTicket},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use r3viewer_wire::{ByteStreamKind, TypedInput, Value};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
};
use std::{
    io::{self, IsTerminal},
    ops::Range,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use unicode_segmentation::UnicodeSegmentation;
const TICK: Duration = Duration::from_millis(25);
const EVENT_CAPACITY: usize = 256;
const HELP: &str = "Flechas / h j k l: mover\nPageUp / PageDown: página; Home / End: extremos\nEnter: abrir celda; columna key: clave completa; r: abrir fila completa\nBackspace: cerrar clave / volver al padre\n/: búsqueda literal; Enter: confirmar; Esc: cancelar editor\nn / N: siguiente / anterior (vuelta al principio/final)\nK: clave completa de columna/campo; L: números de línea; ?: ayuda\nTab / Shift-Tab: Fuente/Estructura TOML; o: span original completo\nq / Esc: salir; Ctrl-C: cancelar (130)";

fn is_cancel(key: &KeyEvent) -> bool {
    key.kind != KeyEventKind::Release
        && (key.code == KeyCode::Char('\u{3}')
            || key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
                && matches!(key.code, KeyCode::Char('c' | 'C')))
}
fn enqueue(
    tx: &mpsc::SyncSender<Event>,
    event: Event,
    cancel: &AtomicBool,
    dropped: &AtomicUsize,
) -> bool {
    if let Event::Key(key) = &event
        && is_cancel(key)
    {
        cancel.store(true, Ordering::Relaxed);
    }
    match tx.try_send(event) {
        Ok(()) => true,
        Err(mpsc::TrySendError::Full(_)) => {
            dropped.fetch_add(1, Ordering::Relaxed);
            true
        }
        Err(mpsc::TrySendError::Disconnected(_)) => false,
    }
}
/// Sole event reader. Ctrl-C is observed even while a long preparation owns the
/// main thread or the bounded queue is full. Drop joins before terminal restore.
struct KeyboardReader {
    rx: mpsc::Receiver<Event>,
    shutdown: Arc<AtomicBool>,
    dropped: Arc<AtomicUsize>,
    error: Arc<Mutex<Option<io::Error>>>,
    thread: Option<JoinHandle<()>>,
}
impl KeyboardReader {
    fn new(cancel: Arc<AtomicBool>) -> io::Result<Self> {
        Self::spawn(cancel, || {
            event::poll(TICK).and_then(|ready| {
                if ready {
                    event::read().map(Some)
                } else {
                    Ok(None)
                }
            })
        })
    }
    fn spawn(
        cancel: Arc<AtomicBool>,
        mut poll: impl FnMut() -> io::Result<Option<Event>> + Send + 'static,
    ) -> io::Result<Self> {
        let (tx, rx) = mpsc::sync_channel(EVENT_CAPACITY);
        let shutdown = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicUsize::new(0));
        let error = Arc::new(Mutex::new(None));
        let stop = shutdown.clone();
        let lost = dropped.clone();
        let failure = error.clone();
        let worker = thread::Builder::new()
            .name("r3viewer-keyboard".into())
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) && !cancel.load(Ordering::Relaxed) {
                    let event = match poll() {
                        Ok(Some(event)) => event,
                        Ok(None) => continue,
                        Err(e) => {
                            *failure.lock().unwrap() = Some(e);
                            break;
                        }
                    };
                    if !enqueue(&tx, event, &cancel, &lost) {
                        break;
                    }
                }
            })?;
        Ok(Self {
            rx,
            shutdown,
            dropped,
            error,
            thread: Some(worker),
        })
    }
    fn error(&self) -> Option<io::Error> {
        self.error.lock().unwrap().take()
    }
}
impl Drop for KeyboardReader {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(worker) = self.thread.take() {
            let _ = worker.join();
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
enum BuildState {
    NotRequested,
    Building { request: u64 },
    Ready,
    Unavailable(String),
}
struct BuildSelection {
    generation: u64,
    sequence: u64,
    state: BuildState,
}
impl BuildSelection {
    fn new(generation: u64) -> Self {
        Self {
            generation,
            sequence: 0,
            state: BuildState::NotRequested,
        }
    }
    fn begin(&mut self) -> Option<u64> {
        if self.state != BuildState::NotRequested {
            return None;
        }
        let request = self.sequence.checked_add(1)?;
        self.sequence = request;
        self.state = BuildState::Building { request };
        Some(request)
    }
    fn cancel_build(&mut self) -> Option<u64> {
        let BuildState::Building { request } = self.state else {
            return None;
        };
        self.state = BuildState::NotRequested;
        Some(request)
    }
    fn current(&self, generation: u64, request: u64) -> bool {
        generation == self.generation && self.state == BuildState::Building { request }
    }
    fn ready(&mut self, generation: u64, request: u64) -> bool {
        if !self.current(generation, request) {
            return false;
        }
        self.state = BuildState::Ready;
        true
    }
    fn unavailable(&mut self, generation: u64, request: u64, diagnostic: String) -> bool {
        if !self.current(generation, request) {
            return false;
        }
        self.state = BuildState::Unavailable(diagnostic);
        true
    }
}
fn structure_toggle(key: &KeyEvent, help: bool, editing: bool) -> bool {
    key.kind == KeyEventKind::Press
        && !help
        && !editing
        && !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        && matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
}
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    top: usize,
    left: usize,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeIdentity {
    Value(usize),
    File(crate::structure_model::NodeId),
    StringStream { pointer: usize, length: usize },
}
struct TextPane {
    model: Arc<SafeTextModel>,
    position: Position,
    hit: Option<Range<usize>>,
    key: bool,
    node_identity: Option<NodeIdentity>,
}
/// Only the most recent viewport is retained. A current-generation fallback
/// dominates every request, including requests replaced while parsing.
#[derive(Default)]
struct SyntaxView {
    generation: u64,
    document: Option<Arc<SafeTextModel>>,
    viewport: Option<Viewport>,
    request: u64,
    spans: Vec<RoleSpan>,
    plain: Option<FallbackReason>,
}
impl SyntaxView {
    fn select(&mut self, document: Option<Arc<SafeTextModel>>) -> bool {
        let same = match (&self.document, &document) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if same {
            return false;
        }
        self.generation = self
            .generation
            .checked_add(1)
            .expect("syntax generation exhausted");
        self.document = document;
        self.viewport = None;
        self.request = 0;
        self.spans.clear();
        self.plain = None;
        true
    }
    fn desired_viewport(&self, top: usize, rows: usize) -> Option<Viewport> {
        let document = self.document.as_ref()?;
        if self.plain.is_some() || top >= document.line_count() || rows == 0 {
            return None;
        }
        Some(Viewport {
            first_line: top as u32,
            line_count: rows.min(256).min(document.line_count() - top) as u16,
        })
    }
    fn requested(&mut self, viewport: Viewport, request: u64) {
        self.viewport = Some(viewport);
        self.request = request;
        self.spans.clear();
    }
    fn apply(&mut self, event: SyntaxEvent) -> bool {
        match event {
            SyntaxEvent::Plain { generation, reason }
                if generation == self.generation && self.document.is_some() =>
            {
                self.spans.clear();
                self.plain = Some(reason);
                true
            }
            SyntaxEvent::Styles {
                generation,
                request,
                viewport,
                spans,
            } if generation == self.generation
                && self.document.is_some()
                && self.plain.is_none()
                && request != 0
                && request == self.request
                && viewport.is_some()
                && viewport == self.viewport =>
            {
                self.spans = spans;
                true
            }
            _ => false,
        }
    }
}
/// Declared before TerminalGuard, so even unwind restores the terminal before
/// joining the controller. stop() only hands cancellation to that controller.
struct SyntaxOwner {
    client: Option<SyntaxClient>,
    shutdown: Option<SyntaxShutdown>,
}
impl SyntaxOwner {
    fn stop(&mut self) {
        if let Some(client) = self.client.take() {
            self.shutdown = Some(client.shutdown());
        }
    }
    fn join(&mut self) -> io::Result<()> {
        self.stop();
        self.shutdown.take().map_or(Ok(()), SyntaxShutdown::join)
    }
}
impl Drop for SyntaxOwner {
    fn drop(&mut self) {
        let _ = self.join();
    }
}
struct StructureOwner {
    client: Option<StructureClient>,
    shutdown: Option<StructureShutdown>,
    resume: Option<ReapTicket>,
    syntax_paused: bool,
}
impl StructureOwner {
    fn stop(&mut self) {
        if let Some(client) = self.client.take() {
            self.shutdown = Some(client.shutdown());
        }
    }
    fn join(&mut self) -> io::Result<()> {
        self.stop();
        self.shutdown.take().map_or(Ok(()), StructureShutdown::join)
    }
}
impl Drop for StructureOwner {
    fn drop(&mut self) {
        let _ = self.join();
    }
}
fn begin_structure(app: &mut App<'_>, owner: &mut StructureOwner, syntax: Option<&SyntaxClient>) {
    let Some(file) = &mut app.file else {
        return;
    };
    let Some(request) = file.build.begin() else {
        return;
    };
    let generation = file.build.generation;
    owner.resume = None;
    let ticket = syntax.map_or_else(ReapTicket::reaped, |client| {
        client.pause_document(app.syntax.generation)
    });
    owner.syntax_paused = true;
    app.syntax.spans.clear();
    app.syntax.viewport = None;
    app.syntax.request = 0;
    app.structure_message =
        "construyendo estructura; Tab vuelve a Fuente y cancela la petición".into();
    if !owner.client.as_ref().is_some_and(|client| {
        client.request(generation, request, StructureMode::Tui, ticket.clone())
    }) {
        let diagnostic = StructureFailure::InvalidIpc.description().to_owned();
        file.build
            .unavailable(generation, request, diagnostic.clone());
        app.structure_message = diagnostic;
        owner.resume = Some(ticket);
    }
}
fn toggle_structure(
    app: &mut App<'_>,
    owner: &mut StructureOwner,
    syntax: Option<&SyntaxClient>,
    cancel: &AtomicBool,
) -> io::Result<()> {
    let Some(file) = &mut app.file else {
        return Ok(());
    };
    match &file.build.state {
        BuildState::NotRequested => begin_structure(app, owner, syntax),
        BuildState::Building { request } => {
            let request = *request;
            // Cancel the exact controller request before retiring its local identity.
            owner.resume = Some(
                owner
                    .client
                    .as_ref()
                    .map_or_else(ReapTicket::reaped, |client| {
                        client.cancel_request(file.build.generation, request)
                    }),
            );
            file.build.cancel_build();
            app.structure_message =
                "construcción cancelada; Tab solicita un nuevo intento explícito".into();
        }
        BuildState::Ready => {
            app.swap_file_domain(cancel)?;
            if app.file_active() && !owner.syntax_paused {
                owner.resume = Some(syntax.map_or_else(ReapTicket::reaped, |client| {
                    client.pause_document(app.syntax.generation)
                }));
                owner.syntax_paused = true;
                app.syntax.spans.clear();
                app.syntax.viewport = None;
                app.syntax.request = 0;
            }
        }
        BuildState::Unavailable(diagnostic) => app.structure_message = diagnostic.clone(),
    }
    Ok(())
}
#[derive(Default)]
struct DomainState {
    pane: Option<TextPane>,
    query: String,
    last_hit: Option<SearchHit>,
    file_hit: Option<FileSearchHit>,
    message: String,
}
struct FileWorkspace {
    source: Arc<String>,
    build: BuildSelection,
    navigator: Option<FileNavigator>,
    active: bool,
    inactive: DomainState,
}
impl FileWorkspace {
    fn new(generation: u64, source: Arc<String>) -> Self {
        Self {
            source,
            build: BuildSelection::new(generation),
            navigator: None,
            active: false,
            inactive: DomainState::default(),
        }
    }
}
struct App<'a> {
    source: Arc<SafeTextModel>,
    typed: Option<Arc<TypedInput>>,
    navigator: Option<Navigator<'a>>,
    pane: Option<TextPane>,
    worker: SearchWorker,
    query: String,
    editor: Option<String>,
    last_hit: Option<SearchHit>,
    help: bool,
    line_numbers: bool,
    message: String,
    searching: bool,
    viewport: (usize, usize),
    syntax: SyntaxView,
    syntax_diagnostic: String,
    file: Option<FileWorkspace>,
    file_hit: Option<FileSearchHit>,
    structure_message: String,
}
fn prepare_error(error: impl std::fmt::Debug) -> io::Error {
    io::Error::other(format!("no se puede preparar vista: {error:?}"))
}
fn copy_key(key: &str, cancel: &AtomicBool) -> io::Result<String> {
    let mut result = String::new();
    let mut start = 0;
    while start < key.len() {
        if cancel.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelado"));
        }
        let mut end = (start + 65536).min(key.len());
        while !key.is_char_boundary(end) {
            end -= 1;
        }
        result.push_str(&key[start..end]);
        start = end;
    }
    Ok(result)
}
impl<'a> App<'a> {
    fn new(
        source: Arc<SafeTextModel>,
        typed: Option<Arc<TypedInput>>,
        borrowed: Option<&'a TypedInput>,
        config: &PresentationConfig,
        cancel: &AtomicBool,
    ) -> io::Result<Self> {
        let navigator = borrowed
            .map(|input| Navigator::new(input, cancel))
            .transpose()
            .map_err(prepare_error)?;
        let mut app = Self {
            source,
            typed,
            navigator,
            pane: None,
            worker: SearchWorker::new(),
            query: String::new(),
            editor: None,
            last_hit: None,
            help: false,
            line_numbers: config.line_numbers,
            message: String::new(),
            searching: false,
            viewport: (1, 1),
            syntax: SyntaxView::default(),
            syntax_diagnostic: String::new(),
            file: None,
            file_hit: None,
            structure_message: String::new(),
        };
        app.refresh_pane(cancel)?;
        Ok(app)
    }
    fn file_active(&self) -> bool {
        self.file.as_ref().is_some_and(|file| file.active)
    }
    fn swap_file_domain(&mut self, cancel: &AtomicBool) -> io::Result<()> {
        self.cancel_pending();
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("sin estructura"))?;
        if file.build.state != BuildState::Ready {
            return Ok(());
        }
        std::mem::swap(&mut self.pane, &mut file.inactive.pane);
        std::mem::swap(&mut self.query, &mut file.inactive.query);
        std::mem::swap(&mut self.last_hit, &mut file.inactive.last_hit);
        std::mem::swap(&mut self.file_hit, &mut file.inactive.file_hit);
        std::mem::swap(&mut self.message, &mut file.inactive.message);
        file.active = !file.active;
        if file.active {
            self.refresh_pane(cancel)?;
        }
        self.reveal_selection();
        Ok(())
    }
    fn install_structure(
        &mut self,
        generation: u64,
        request: u64,
        structure: Arc<crate::structure_model::Structure>,
        cancel: &AtomicBool,
    ) -> io::Result<bool> {
        let Some(file) = &mut self.file else {
            return Ok(false);
        };
        if structure.generation() != generation
            || !Arc::ptr_eq(&file.source, &structure.source_handle())
            || !file.build.ready(generation, request)
        {
            return Ok(false);
        }
        let duplicate_diagnostic = structure.duplicate_diagnostic();
        file.navigator = Some(FileNavigator::new(structure));
        self.structure_message.clear();
        if self.editor.is_none() && !self.help {
            self.swap_file_domain(cancel)?;
        } else {
            self.structure_message = "estructura disponible; Tab alterna la vista".into();
        }
        if let Some(diagnostic) = duplicate_diagnostic {
            self.structure_message = diagnostic;
        }
        Ok(true)
    }
    fn file_key(
        &mut self,
        code: KeyCode,
        rows: usize,
        columns: usize,
        cancel: &AtomicBool,
    ) -> io::Result<bool> {
        if !self.file_active() {
            return Ok(false);
        }
        if code == KeyCode::Char('o') {
            let nav = self.file.as_ref().unwrap().navigator.as_ref().unwrap();
            let span = nav.selected_location();
            if nav.structure().generation() != self.file.as_ref().unwrap().build.generation
                || span.is_none()
            {
                self.message = "el nodo no tiene un span original contiguo".into();
                return Ok(true);
            }
            let range = self.source.visible_range(span.unwrap().range());
            self.swap_file_domain(cancel)?;
            if let Some(pane) = &mut self.pane {
                pane.position.top = pane.model.line_for_offset(range.start);
                pane.position.left = offset_cell(&pane.model, range.start, cancel)?;
                pane.hit = Some(range);
            }
            self.message = "span original completo; consulta de Fuente conservada".into();
            return Ok(true);
        }
        let recognized = matches!(
            code,
            KeyCode::Enter
                | KeyCode::Backspace
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::Char('K' | 'r' | 'h' | 'j' | 'k' | 'l')
        );
        if !recognized {
            return Ok(false);
        }
        let mut changed = false;
        if code == KeyCode::Backspace {
            changed = self
                .file
                .as_mut()
                .unwrap()
                .navigator
                .as_mut()
                .unwrap()
                .back();
            if changed {
                self.refresh_pane(cancel)?;
            }
        } else if matches!(code, KeyCode::Enter | KeyCode::Char('K' | 'r')) {
            if self.pane.is_none() {
                let nav = self.file.as_mut().unwrap().navigator.as_mut().unwrap();
                let cursor = nav.current().cursor;
                let column = match code {
                    KeyCode::Char('K') => 0,
                    KeyCode::Char('r') => 1,
                    _ => cursor.column,
                };
                changed = nav
                    .enter(cursor.row, column, cancel)
                    .map_err(prepare_error)?;
                if changed {
                    self.refresh_pane(cancel)?;
                }
            }
        } else if let Some(pane) = &mut self.pane {
            let before = pane.position;
            move_text(
                &mut pane.position,
                code,
                pane.model.line_count(),
                pane.model.max_width(),
                rows,
                columns,
            );
            let cursor = &mut self
                .file
                .as_mut()
                .unwrap()
                .navigator
                .as_mut()
                .unwrap()
                .current_mut()
                .cursor;
            cursor.top = pane.position.top;
            cursor.left = pane.position.left;
            changed = before != pane.position;
        } else {
            let nav = self.file.as_mut().unwrap().navigator.as_mut().unwrap();
            let count = nav.view().row_count();
            let cursor = &mut nav.current_mut().cursor;
            let before = *cursor;
            match code {
                KeyCode::Up | KeyCode::Char('k') => cursor.row = cursor.row.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => {
                    cursor.row = cursor.row.saturating_add(1).min(count.saturating_sub(1))
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    cursor.column = cursor.column.saturating_sub(1)
                }
                KeyCode::Right | KeyCode::Char('l') => cursor.column = (cursor.column + 1).min(1),
                KeyCode::PageUp => cursor.row = cursor.row.saturating_sub(rows.max(1)),
                KeyCode::PageDown => {
                    cursor.row = cursor
                        .row
                        .saturating_add(rows.max(1))
                        .min(count.saturating_sub(1))
                }
                KeyCode::Home => {
                    cursor.row = 0;
                    cursor.column = 0;
                }
                KeyCode::End => cursor.row = count.saturating_sub(1),
                _ => (),
            }
            nav.frame(rows, columns);
            changed = before != nav.current().cursor;
        }
        if changed {
            self.cancel_pending();
        }
        self.reveal_selection();
        Ok(true)
    }
    fn syntax_document(&self) -> Option<Arc<SafeTextModel>> {
        if self.file_active() {
            return None;
        }
        let pane = self.pane.as_ref()?;
        if pane.key {
            return None;
        }
        if self.typed.is_none()
            || self.navigator.as_ref().is_some_and(|nav| {
                matches!(
                    nav.current().node(),
                    DataRef::Value(Value::String(_))
                        | DataRef::ByteStream {
                            kind: ByteStreamKind::String,
                            ..
                        }
                )
            })
        {
            Some(pane.model.clone())
        } else {
            None
        }
    }
    fn sync_syntax(&mut self, client: &SyntaxClient, language: &str) {
        if self.syntax.select(self.syntax_document()) {
            self.syntax_diagnostic.clear();
            if let Some(document) = &self.syntax.document {
                client.set_document(
                    document.original_handle(),
                    language.to_owned(),
                    self.syntax.generation,
                );
            } else {
                client.clear_document(self.syntax.generation);
            }
        }
    }
    fn request_syntax(&mut self, client: &SyntaxClient, rows: usize) {
        let top = self.pane.as_ref().map_or(0, |pane| pane.position.top);
        if let Some(viewport) = self.syntax.desired_viewport(top, rows)
            && self.syntax.viewport != Some(viewport)
        {
            let request = client.request_viewport(self.syntax.generation, viewport);
            self.syntax.requested(viewport, request);
        }
    }
    fn apply_syntax(&mut self, event: SyntaxEvent) -> bool {
        let changed = self.syntax.apply(event);
        if changed && let Some(reason) = self.syntax.plain {
            self.syntax_diagnostic = fallback_diagnostic(reason);
        }
        changed
    }
    fn refresh_pane(&mut self, cancel: &AtomicBool) -> io::Result<()> {
        if self.file_active() {
            let nav = self.file.as_ref().unwrap().navigator.as_ref().unwrap();
            let node = nav.view().node();
            if node.kind().is_container() {
                self.pane = None;
                return Ok(());
            }
            let identity = Some(NodeIdentity::File(node.id()));
            if self
                .pane
                .as_ref()
                .is_some_and(|pane| pane.node_identity == identity)
            {
                return Ok(());
            }
            let raw = file_leaf_text(node.leaf().expect("validated leaf"), cancel)
                .map_err(prepare_error)?;
            let model = SafeTextModel::new(raw, cancel).map_err(prepare_error)?;
            let cursor = nav.current().cursor;
            self.pane = Some(TextPane {
                model: Arc::new(model),
                position: Position {
                    top: cursor.top,
                    left: cursor.left,
                },
                hit: None,
                key: nav
                    .current()
                    .path
                    .0
                    .last()
                    .is_some_and(|step| step.part == FilePart::Key),
                node_identity: identity,
            });
            return Ok(());
        }
        if let Some(nav) = &self.navigator {
            let node = nav.current().node();
            let identity = match node {
                DataRef::Value(value) => Some(NodeIdentity::Value(value as *const Value as usize)),
                DataRef::ByteStream {
                    kind: ByteStreamKind::String,
                    bytes,
                } => Some(NodeIdentity::StringStream {
                    pointer: bytes.as_ptr() as usize,
                    length: bytes.len(),
                }),
                _ => None,
            };
            if let Some(pane) = &mut self.pane
                && !pane.key
                && identity.is_some()
                && pane.node_identity == identity
            {
                let cursor = nav.current().cursor;
                pane.position = Position {
                    top: cursor.top,
                    left: cursor.left,
                };
                return Ok(());
            }
            self.pane = if node.is_container() {
                None
            } else {
                let raw = primitive_text(node, cancel).map_err(prepare_error)?;
                let model = SafeTextModel::new(raw, cancel).map_err(prepare_error)?;
                let cursor = nav.current().cursor;
                Some(TextPane {
                    model: Arc::new(model),
                    position: Position {
                        top: cursor.top,
                        left: cursor.left,
                    },
                    hit: None,
                    key: false,
                    node_identity: identity,
                })
            };
        } else {
            self.pane = Some(TextPane {
                model: self.source.clone(),
                position: Position::default(),
                hit: None,
                key: false,
                node_identity: None,
            });
        }
        Ok(())
    }
    fn open_key(&mut self, key: &str, cancel: &AtomicBool) -> io::Result<()> {
        let model = SafeTextModel::new(copy_key(key, cancel)?, cancel).map_err(prepare_error)?;
        self.pane = Some(TextPane {
            model: Arc::new(model),
            position: Position::default(),
            hit: None,
            key: true,
            node_identity: None,
        });
        Ok(())
    }
    fn search(&mut self, direction: Direction) {
        if self.query.is_empty() {
            self.message = "consulta vacía".into();
            return;
        }
        let query = self.query.clone();
        let previous = self.last_hit.clone();
        if self.file_active() {
            let structure = self
                .file
                .as_ref()
                .unwrap()
                .navigator
                .as_ref()
                .unwrap()
                .structure()
                .clone();
            let previous = self.file_hit.clone();
            self.worker.submit(move |cancel| {
                find_structure(&structure, &query, previous.as_ref(), direction, cancel)
            });
        } else if let Some(input) = &self.typed {
            let input = input.clone();
            self.worker.submit(move |cancel| {
                find_data(&input, &query, previous.as_ref(), direction, cancel)
            });
        } else {
            let source = self.source.clone();
            self.worker.submit(move |cancel| {
                let offset = previous.as_ref().map_or_else(
                    || {
                        if direction == Direction::Forward {
                            0
                        } else {
                            source.visible().len()
                        }
                    },
                    |hit| {
                        if direction == Direction::Forward {
                            hit.range.end
                        } else {
                            hit.range.start
                        }
                    },
                );
                match find_visible(source.visible(), &query, offset, direction, cancel) {
                    Ok(Some(range)) => SearchOutcome::Found(SearchHit {
                        path: None,
                        key: false,
                        visit_index: 0,
                        range,
                    }),
                    Ok(None) => SearchOutcome::NotFound,
                    Err(e) if cancel.load(Ordering::Relaxed) => {
                        let _ = e;
                        SearchOutcome::Cancelled
                    }
                    Err(e) => SearchOutcome::Error(e),
                }
            });
        }
        self.searching = true;
        self.message = "buscando…".into();
    }
    fn apply_search(&mut self, outcome: SearchOutcome, cancel: &AtomicBool) -> io::Result<()> {
        self.searching = false;
        match outcome {
            SearchOutcome::Found(hit) => {
                if let (Some(nav), Some(path)) = (&mut self.navigator, &hit.path) {
                    if hit.key {
                        let mut parent = path.clone();
                        let Some(PathMember::Key(key)) = parent.0.pop() else {
                            return Err(io::Error::other(
                                "coincidencia de clave con ruta inválida",
                            ));
                        };
                        nav.jump(&parent, cancel).map_err(prepare_error)?;
                        let record = nav
                            .current()
                            .node()
                            .record()
                            .ok_or_else(|| io::Error::other("clave sin registro padre"))?;
                        let row = record
                            .iter()
                            .position(|(name, _)| name == &key)
                            .ok_or_else(|| io::Error::other("clave no encontrada"))?;
                        nav.current_mut().cursor.row = row;
                        nav.current_mut().cursor.column = 0;
                        self.open_key(&key, cancel)?;
                    } else {
                        nav.jump(path, cancel).map_err(prepare_error)?;
                        self.refresh_pane(cancel)?;
                    }
                }
                if let Some(pane) = &mut self.pane {
                    pane.position.top = pane.model.line_for_offset(hit.range.start);
                    pane.position.left = offset_cell(&pane.model, hit.range.start, cancel)?;
                    pane.hit = Some(hit.range.clone());
                }
                self.last_hit = Some(hit);
                self.message = "coincidencia".into();
                self.help = false;
            }
            SearchOutcome::FileFound(hit) => {
                if !self.file_active() {
                    return Ok(());
                }
                self.file
                    .as_mut()
                    .unwrap()
                    .navigator
                    .as_mut()
                    .unwrap()
                    .jump(&hit.path, cancel)
                    .map_err(prepare_error)?;
                self.refresh_pane(cancel)?;
                if let Some(pane) = &mut self.pane {
                    pane.position.top = pane.model.line_for_offset(hit.range.start);
                    pane.position.left = offset_cell(&pane.model, hit.range.start, cancel)?;
                    pane.hit = Some(hit.range.clone());
                }
                self.file_hit = Some(hit);
                self.message = "coincidencia en Estructura".into();
                self.help = false;
            }
            SearchOutcome::NotFound => self.message = "sin coincidencias".into(),
            SearchOutcome::Cancelled => self.message = "búsqueda cancelada".into(),
            SearchOutcome::Error(e) => self.message = e,
        }
        Ok(())
    }
    fn cancel_pending(&mut self) {
        if self.searching {
            self.worker.cancel();
            self.searching = false;
            self.message = "búsqueda cancelada al navegar".into();
        }
    }
    fn set_viewport(&mut self, rows: usize, columns: usize) {
        if self.viewport != (rows, columns) {
            self.viewport = (rows, columns);
            self.reveal_selection();
        }
    }
    fn reveal_selection(&mut self) {
        if self.pane.is_none() && self.file_active() {
            self.file
                .as_mut()
                .unwrap()
                .navigator
                .as_mut()
                .unwrap()
                .frame(self.viewport.0, self.viewport.1);
            return;
        }
        if self.pane.is_none()
            && let Some(nav) = &mut self.navigator
        {
            frame_data(nav, self.viewport.0, self.viewport.1);
        }
    }
    fn key(
        &mut self,
        key: KeyEvent,
        rows: usize,
        columns: usize,
        cancel: &AtomicBool,
    ) -> io::Result<bool> {
        if key.kind == KeyEventKind::Release {
            return Ok(false);
        }
        self.set_viewport(rows, columns);
        if let Some(editor) = &mut self.editor {
            match key.code {
                KeyCode::Esc => {
                    self.editor = None;
                    self.message = "edición cancelada".into();
                }
                KeyCode::Enter => {
                    self.query = editor.clone();
                    self.editor = None;
                    self.last_hit = None;
                    self.file_hit = None;
                    self.search(Direction::Forward);
                }
                KeyCode::Backspace => {
                    editor.pop();
                }
                KeyCode::Char(ch)
                    if !ch.is_control()
                        && (!key
                            .modifiers
                            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                            || key
                                .modifiers
                                .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)) =>
                {
                    if editor.len() + ch.len_utf8() <= QUERY_LIMIT {
                        editor.push(ch);
                    } else {
                        self.message = format!("consulta limitada a {QUERY_LIMIT} bytes");
                    }
                }
                _ => {}
            }
            return Ok(false);
        }
        // AltGr text must not trigger a command after leaving the editor.
        // Ctrl-C is handled separately by the sole event reader.
        if matches!(key.code, KeyCode::Char(_))
            && key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return Ok(false);
        }
        if !self.help && self.file_key(key.code, rows, columns, cancel)? {
            return Ok(false);
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('?') => {
                self.cancel_pending();
                self.help = !self.help;
            }
            KeyCode::Char('/') => {
                self.worker.cancel();
                self.searching = false;
                self.editor = Some(self.query.clone());
            }
            KeyCode::Char('n') => self.search(Direction::Forward),
            KeyCode::Char('N') => self.search(Direction::Backward),
            KeyCode::Char('L') => self.line_numbers = !self.line_numbers,
            KeyCode::Char('K') => {
                let selected = self.navigator.as_ref().and_then(|nav| {
                    let view = nav.current();
                    if view.is_table() {
                        view.column_key(view.cursor.column)
                    } else {
                        match view.cell(view.cursor.row, 0) {
                            Some(Cell::Key(key)) => Some(key),
                            _ => None,
                        }
                    }
                });
                if let Some(key) = selected {
                    self.cancel_pending();
                    self.open_key(key, cancel)?;
                }
            }
            KeyCode::Char('r') if self.pane.is_none() => {
                let entered = if let Some(nav) = &mut self.navigator {
                    nav.enter_row(nav.current().cursor.row, cancel)
                        .map_err(prepare_error)?
                } else {
                    false
                };
                if entered {
                    self.cancel_pending();
                    self.refresh_pane(cancel)?;
                }
            }
            KeyCode::Backspace => {
                let overlay = self.pane.as_ref().is_some_and(|p| p.key);
                let can_back = self
                    .navigator
                    .as_ref()
                    .is_some_and(|nav| nav.breadcrumbs().nth(1).is_some());
                if overlay || can_back {
                    self.cancel_pending();
                    if !overlay {
                        self.navigator.as_mut().unwrap().back();
                    }
                    self.refresh_pane(cancel)?;
                }
            }
            KeyCode::Enter if self.pane.is_none() => {
                let selected = self.navigator.as_ref().and_then(|nav| {
                    let cursor = nav.current().cursor;
                    nav.current()
                        .cell(cursor.row, cursor.column)
                        .map(|cell| (cursor.row, cursor.column, cell))
                });
                if let Some((row, column, cell)) =
                    selected.filter(|(_, _, cell)| !matches!(cell, Cell::Missing))
                {
                    self.cancel_pending();
                    if let Cell::Key(key) = cell {
                        self.open_key(key, cancel)?;
                    } else if self
                        .navigator
                        .as_mut()
                        .unwrap()
                        .enter(row, column, cancel)
                        .map_err(prepare_error)?
                    {
                        self.refresh_pane(cancel)?;
                    }
                }
            }
            code => {
                let moved = if let Some(pane) = &mut self.pane {
                    let before = pane.position;
                    move_text(
                        &mut pane.position,
                        code,
                        pane.model.line_count(),
                        pane.model.max_width(),
                        rows,
                        columns,
                    );
                    if !pane.key
                        && let Some(nav) = &mut self.navigator
                    {
                        nav.current_mut().cursor.top = pane.position.top;
                        nav.current_mut().cursor.left = pane.position.left;
                    }
                    before != pane.position
                } else if let Some(nav) = &mut self.navigator {
                    let before = nav.current().cursor;
                    move_data(nav, code, rows, columns);
                    before != nav.current().cursor
                } else {
                    false
                };
                if moved {
                    self.cancel_pending();
                }
            }
        }
        self.reveal_selection();
        Ok(false)
    }
}
fn offset_cell(model: &SafeTextModel, offset: usize, cancel: &AtomicBool) -> io::Result<usize> {
    let range = model.line_range(model.line_for_offset(offset)).unwrap();
    let mut column = 0;
    for (index, (byte, grapheme)) in model.visible()[range.clone()]
        .grapheme_indices(true)
        .enumerate()
    {
        if index.is_multiple_of(4096) && cancel.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelado"));
        }
        if range.start + byte + grapheme.len() > offset {
            break;
        }
        column += cell_width(grapheme, column);
    }
    Ok(column)
}
fn move_text(
    pos: &mut Position,
    code: KeyCode,
    lines: usize,
    max_left: usize,
    rows: usize,
    columns: usize,
) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => pos.top = pos.top.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => {
            pos.top = pos.top.saturating_add(1).min(lines.saturating_sub(1))
        }
        KeyCode::Left | KeyCode::Char('h') => pos.left = pos.left.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => pos.left = pos.left.saturating_add(1).min(max_left),
        KeyCode::PageUp => pos.top = pos.top.saturating_sub(rows.max(1)),
        KeyCode::PageDown => {
            pos.top = pos
                .top
                .saturating_add(rows.max(1))
                .min(lines.saturating_sub(1))
        }
        KeyCode::Home => *pos = Position::default(),
        KeyCode::End => pos.top = lines.saturating_sub(rows.max(1)),
        _ => {
            let _ = columns;
        }
    }
}
fn move_data(nav: &mut Navigator<'_>, code: KeyCode, rows: usize, columns: usize) {
    let view = nav.current_mut();
    let count = view.row_count();
    let cols = view.column_count();
    match code {
        KeyCode::Up | KeyCode::Char('k') => view.cursor.row = view.cursor.row.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => {
            view.cursor.row = view
                .cursor
                .row
                .saturating_add(1)
                .min(count.saturating_sub(1))
        }
        KeyCode::Left | KeyCode::Char('h') => {
            view.cursor.column = view.cursor.column.saturating_sub(1)
        }
        KeyCode::Right | KeyCode::Char('l') => {
            view.cursor.column = view
                .cursor
                .column
                .saturating_add(1)
                .min(cols.saturating_sub(1))
        }
        KeyCode::PageUp => view.cursor.row = view.cursor.row.saturating_sub(rows.max(1)),
        KeyCode::PageDown => {
            view.cursor.row = view
                .cursor
                .row
                .saturating_add(rows.max(1))
                .min(count.saturating_sub(1))
        }
        KeyCode::Home => {
            view.cursor.row = 0;
            view.cursor.column = 0;
        }
        KeyCode::End => view.cursor.row = count.saturating_sub(1),
        _ => {}
    }
    frame_data(nav, rows, columns);
}
fn frame_data(nav: &mut Navigator<'_>, rows: usize, columns: usize) {
    let view = nav.current_mut();
    view.clamp_cursor();
    let rows = rows.max(1);
    if view.cursor.row < view.cursor.top {
        view.cursor.top = view.cursor.row;
    }
    if view.cursor.row >= view.cursor.top.saturating_add(rows) {
        view.cursor.top = view.cursor.row.saturating_sub(rows - 1);
    }
    let visible_cols = if view.is_table() {
        (columns / 24).max(1)
    } else if columns == 1 {
        1
    } else {
        2
    };
    if view.cursor.column < view.cursor.left {
        view.cursor.left = view.cursor.column;
    }
    if view.cursor.column >= view.cursor.left.saturating_add(visible_cols) {
        view.cursor.left = view.cursor.column + 1 - visible_cols;
    }
}
#[derive(Default, Debug)]
struct RenderStats {
    rows: usize,
    cells: usize,
    graphemes: usize,
}
fn label(text: &str, width: usize) -> String {
    let head: String = text
        .chars()
        .take(width.saturating_mul(4))
        .map(|ch| if ch.is_control() { '�' } else { ch })
        .collect();
    slice_cells(&head, 0, width)
}
struct TextDecoration<'a> {
    hit: Option<&'a Range<usize>>,
    syntax: &'a [RoleSpan],
}
fn syntax_role(role: SyntaxRole) -> Role {
    match role {
        SyntaxRole::Value => Role::Value,
        SyntaxRole::Secondary => Role::Secondary,
        SyntaxRole::Accent => Role::Accent,
        SyntaxRole::Success => Role::Success,
        SyntaxRole::Process => Role::Process,
        SyntaxRole::Heading => Role::Heading,
    }
}
fn text_line(
    model: &SafeTextModel,
    line: usize,
    window: Range<usize>,
    decoration: TextDecoration<'_>,
    config: &PresentationConfig,
    cancel: &AtomicBool,
    stats: &mut RenderStats,
) -> Line<'static> {
    let Some(range) = model.line_range(line) else {
        return Line::default();
    };
    let left = window.start;
    let right = window.end;
    let (start, mut column) = model.viewport_start(line, left).unwrap();
    let mut spans = Vec::new();
    let mut provenance = model.provenance_cursor(start);
    let mut role_index = None;
    for (index, (byte, grapheme)) in model.visible()[start..range.end]
        .grapheme_indices(true)
        .enumerate()
    {
        if column >= right || index.is_multiple_of(4096) && cancel.load(Ordering::Relaxed) {
            break;
        }
        stats.graphemes += 1;
        let end = column.saturating_add(cell_width(grapheme, column));
        if end > column && end > left {
            let source_range = start + byte..start + byte + grapheme.len();
            let matched = decoration
                .hit
                .is_some_and(|hit| hit.start < source_range.end && hit.end > source_range.start);
            let raw = provenance.source_range(source_range);
            let index = role_index.get_or_insert_with(|| {
                decoration
                    .syntax
                    .partition_point(|span| span.end as usize <= raw.start)
            });
            while *index < decoration.syntax.len()
                && decoration.syntax[*index].end as usize <= raw.start
            {
                *index += 1;
            }
            // The first original scalar determines the whole visible cluster.
            // Later combining/ZWJ scalars never supply a missing initial role.
            let role = decoration
                .syntax
                .get(*index)
                .filter(|span| span.start as usize <= raw.start)
                .map_or(Role::Value, |span| syntax_role(span.role));
            let mut style = config.style(role);
            if matched {
                style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
            }
            let piece = if grapheme == "\t" || column < left || end > right {
                " ".repeat(end.min(right) - column.max(left))
            } else {
                grapheme.to_owned()
            };
            spans.push(Span::styled(piece, style));
        }
        column = end;
    }
    Line::from(spans)
}
fn render(
    frame: &mut Frame<'_>,
    app: &App<'_>,
    title: &str,
    config: &PresentationConfig,
    dropped: usize,
    cancel: &AtomicBool,
) -> RenderStats {
    let mut stats = RenderStats::default();
    let area = frame.area();
    let width = area.width.saturating_sub(1);
    let content_height = area.height.saturating_sub(2);
    if width == 0 || area.height == 0 {
        return stats;
    }
    let header = if app.file_active() {
        let nav = app.file.as_ref().unwrap().navigator.as_ref().unwrap();
        format!(
            "{} | Estructura {}",
            label(title, usize::from(width) / 2),
            nav.current().path.label(usize::from(width) / 2)
        )
    } else if let Some(nav) = &app.navigator {
        format!(
            "{} | {}{}",
            label(title, usize::from(width) / 2),
            nav.current().path.label(usize::from(width) / 2),
            if app.pane.as_ref().is_some_and(|p| p.key) {
                " | clave"
            } else {
                ""
            }
        )
    } else {
        label(
            &format!(
                "{}{}",
                title,
                if app.file.is_some() { " | Fuente" } else { "" }
            ),
            usize::from(width),
        )
    };
    frame.render_widget(
        Paragraph::new(header).style(config.style(Role::Heading)),
        Rect::new(area.x, area.y, width, 1),
    );
    let body = Rect::new(area.x, area.y.saturating_add(1), width, content_height);
    if app.help {
        frame.render_widget(
            Paragraph::new(HELP).style(config.style(Role::Secondary)),
            body,
        );
    } else if let Some(pane) = &app.pane {
        let numbers = if app.line_numbers {
            pane.model.line_count().to_string().len() + 2
        } else {
            0
        };
        for row in 0..usize::from(content_height) {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let line = pane.position.top + row;
            if line >= pane.model.line_count() {
                break;
            }
            stats.rows += 1;
            let number_width = numbers.min(usize::from(width));
            if number_width > 0 {
                frame.render_widget(
                    Paragraph::new(format!(
                        "{:>w$} ",
                        line + 1,
                        w = number_width.saturating_sub(1)
                    ))
                    .style(config.style(Role::Secondary)),
                    Rect::new(body.x, body.y + row as u16, number_width as u16, 1),
                );
            }
            let line = text_line(
                &pane.model,
                line,
                pane.position.left
                    ..pane
                        .position
                        .left
                        .saturating_add(usize::from(width) - number_width),
                TextDecoration {
                    hit: pane.hit.as_ref(),
                    syntax: if app.syntax.plain.is_none()
                        && app
                            .syntax
                            .document
                            .as_ref()
                            .is_some_and(|document| Arc::ptr_eq(document, &pane.model))
                    {
                        &app.syntax.spans
                    } else {
                        &[]
                    },
                },
                config,
                cancel,
                &mut stats,
            );
            frame.render_widget(
                Paragraph::new(line),
                Rect::new(
                    body.x + number_width as u16,
                    body.y + row as u16,
                    width - number_width as u16,
                    1,
                ),
            );
        }
    } else if app.file_active() {
        let nav = app.file.as_ref().unwrap().navigator.as_ref().unwrap();
        let view = nav.view();
        let first = if width == 1 {
            view.state.cursor.column
        } else {
            0
        };
        let column_width = if width == 1 {
            1
        } else {
            (usize::from(width) / 3).max(1)
        };
        for column in first..if width == 1 { first + 1 } else { 2 } {
            let x = if width == 1 { 0 } else { column * column_width };
            let cw = if column == 0 || width == 1 {
                column_width
            } else {
                usize::from(width) - x
            };
            frame.render_widget(
                Paragraph::new(view.column_name(column).unwrap_or(""))
                    .style(config.style(Role::Secondary)),
                Rect::new(body.x + x as u16, body.y, cw as u16, 1),
            );
        }
        let end = view.row_count().min(
            view.state
                .cursor
                .top
                .saturating_add(usize::from(content_height.saturating_sub(1))),
        );
        for row in view.state.cursor.top.min(end)..end {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            stats.rows += 1;
            let y = body.y + 1 + (row - view.state.cursor.top) as u16;
            for column in first..if width == 1 { first + 1 } else { 2 } {
                let x = if width == 1 { 0 } else { column * column_width };
                let cw = if column == 0 || width == 1 {
                    column_width
                } else {
                    usize::from(width) - x
                };
                if let Some(cell) = view.cell(row, column) {
                    stats.cells += 1;
                    let mut style = config.style(Role::Value);
                    if row == view.state.cursor.row && column == view.state.cursor.column {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    frame.render_widget(
                        Paragraph::new(label(&cell.summary(cw.saturating_mul(2)), cw)).style(style),
                        Rect::new(body.x + x as u16, y, cw as u16, 1),
                    );
                }
            }
        }
        if view.row_count() == 0 && content_height > 1 {
            frame.render_widget(
                Paragraph::new(InspectionRef::File(view.node()).summary(usize::from(width)))
                    .style(config.style(Role::Secondary)),
                Rect::new(body.x, body.y + 1, width, 1),
            );
        }
    } else if let Some(nav) = &app.navigator {
        let view = nav.current();
        let table = view.is_table();
        let cols = view.column_count();
        let first = if table {
            view.cursor.left
        } else if width == 1 {
            view.cursor.column
        } else {
            0
        };
        let visible_cols = if table {
            (usize::from(width) / 24).max(1)
        } else if width == 1 {
            1
        } else {
            cols
        };
        let row_width = if table {
            24usize.min(usize::from(width))
        } else {
            (usize::from(width) / 3).max(1)
        };
        for col in first..cols.min(first.saturating_add(visible_cols)) {
            let x = (col - first) * row_width;
            if x >= usize::from(width) {
                break;
            }
            let cw = if width == 1 {
                1
            } else if table || col == 0 {
                row_width
            } else {
                usize::from(width) - x
            };
            frame.render_widget(
                Paragraph::new(label(view.column_name(col).unwrap_or(""), cw))
                    .style(config.style(Role::Secondary)),
                Rect::new(body.x + x as u16, body.y, cw as u16, 1),
            );
        }
        let available_rows = usize::from(content_height.saturating_sub(1));
        for row in view.rows(view.cursor.top..view.cursor.top.saturating_add(available_rows)) {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            stats.rows += 1;
            let y = body.y + 1 + (row.index - view.cursor.top) as u16;
            for col in first..cols.min(first.saturating_add(visible_cols)) {
                let x = (col - first) * row_width;
                if x >= usize::from(width) {
                    break;
                }
                let cw = if width == 1 {
                    1
                } else if table || col == 0 {
                    row_width
                } else {
                    usize::from(width) - x
                };
                let Some(cell) = row.cell(col) else {
                    continue;
                };
                stats.cells += 1;
                let mut style = if matches!(cell, Cell::Missing) {
                    config.style(Role::Secondary)
                } else {
                    config.style(Role::Value)
                };
                if row.index == view.cursor.row && col == view.cursor.column {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                frame.render_widget(
                    Paragraph::new(label(&cell.summary(cw.saturating_mul(2)), cw)).style(style),
                    Rect::new(body.x + x as u16, y, cw as u16, 1),
                );
            }
        }
        if view.row_count() == 0 {
            frame.render_widget(
                Paragraph::new(view.node().summary(usize::from(width)))
                    .style(config.style(Role::Secondary)),
                Rect::new(
                    body.x,
                    body.y.saturating_add(1),
                    width,
                    1.min(content_height.saturating_sub(1)),
                ),
            );
        }
    }
    let status = if let Some(editor) = &app.editor {
        format!(
            "{} /{} | Enter confirmar · Esc cancelar",
            if dropped > 0 {
                format!("cola: {dropped} descartados")
            } else {
                String::new()
            },
            label(editor, usize::from(width))
        )
    } else {
        format!(
            "{}{}{} | / buscar · n/N · Enter abrir · Backspace volver · ? · q",
            if dropped > 0 {
                format!("cola: {dropped} descartados | ")
            } else {
                String::new()
            },
            if app.searching { "buscando… " } else { "" },
            label(
                &format!(
                    "{}{}{} | {}",
                    app.message,
                    if app.message.is_empty() || app.syntax_diagnostic.is_empty() {
                        ""
                    } else {
                        " | "
                    },
                    app.syntax_diagnostic,
                    app.structure_message
                ),
                usize::from(width) / 2
            )
        )
    };
    if area.height > 1 {
        frame.render_widget(
            Paragraph::new(label(&status, usize::from(width))).style(config.style(Role::Secondary)),
            Rect::new(area.x, area.y + area.height - 1, width, 1),
        );
    }
    stats
}

/// true means cancelled (exit130); normal q/Escape returns false. The keyboard
/// reader and search worker are always joined before exact terminal restoration.
// The session contract keeps source, presentation, and cancellation explicit.
#[allow(clippy::too_many_arguments)]
pub fn run(
    source_text: Arc<SafeTextModel>,
    typed: Option<Arc<TypedInput>>,
    title: &str,
    config: &PresentationConfig,
    selection: &SyntaxSelection,
    structure_selection: &StructureSelection,
    initial_structure: bool,
    cancelled: Arc<AtomicBool>,
) -> io::Result<bool> {
    if cancelled.load(Ordering::Relaxed) {
        return Ok(true);
    }
    let stdout = io::stdout();
    if !stdout.is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "la sesión requiere stdout terminal",
        ));
    }
    crossterm::style::force_color_output(config.coloured);
    let language = match selection {
        SyntaxSelection::Language(language) if config.coloured => Some(language.as_str()),
        _ => None,
    };
    let client = language.map(|_| SyntaxClient::new(SyntaxLimits::default(), cancelled.clone()));
    let mut syntax_diagnostic = match selection {
        SyntaxSelection::Plain { diagnostic } => diagnostic.clone().unwrap_or_default(),
        _ => String::new(),
    };
    let mut syntax_owner = SyntaxOwner {
        client: match client {
            Some(Ok(client)) => Some(client),
            Some(Err(_)) => {
                syntax_diagnostic = fallback_diagnostic(FallbackReason::WorkerExited);
                None
            }
            None => None,
        },
        shutdown: None,
    };
    let mut structure_diagnostic = structure_selection.diagnostic().unwrap_or("").to_owned();
    let mut structure_owner = StructureOwner {
        client: if structure_selection.format().is_some() {
            match StructureClient::new(cancelled.clone(), structure_selection.format().unwrap()) {
                Ok(client) => {
                    client.set_document(source_text.original_handle(), 1);
                    Some(client)
                }
                Err(_) => {
                    structure_diagnostic = StructureFailure::WorkerExited.description().into();
                    None
                }
            }
        } else {
            None
        },
        shutdown: None,
        resume: None,
        syntax_paused: false,
    };
    let mut guard = TerminalGuard::enter(&stdout, control_terminal()?)?;
    // Declaration order and the inner scope keep reader/worker alive during
    // cancellable preparation, and drop both before the owning guard.
    let result = (|| {
        let keyboard = KeyboardReader::new(cancelled.clone())?;
        let inspect_typed = if structure_selection.format().is_some() {
            None
        } else {
            typed.clone()
        };
        let mut app = App::new(
            source_text,
            inspect_typed.clone(),
            inspect_typed.as_deref(),
            config,
            &cancelled,
        )?;
        app.syntax_diagnostic = syntax_diagnostic;
        app.structure_message = structure_diagnostic;
        if structure_selection.format().is_some() {
            app.file = Some(FileWorkspace::new(1, app.source.original_handle()));
            if structure_owner.client.is_none() {
                app.file.as_mut().unwrap().build.state =
                    BuildState::Unavailable(app.structure_message.clone());
            }
        }
        if let Some(client) = &syntax_owner.client {
            app.sync_syntax(client, language.unwrap());
        }
        if initial_structure {
            begin_structure(&mut app, &mut structure_owner, syntax_owner.client.as_ref());
        }
        let mut terminal = Terminal::new(CrosstermBackend::new(&mut guard.output))?;
        let mut redraw = true;
        let mut last_dropped = 0;
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Ok(true);
            }
            if let Some(error) = keyboard.error() {
                return Err(error);
            }
            if let Some(outcome) = app.worker.poll() {
                app.apply_search(outcome, &cancelled)?;
                redraw = true;
            }
            if let Some(client) = &mut structure_owner.client
                && let Some(event) = client.poll()
            {
                match event {
                    StructureEvent::Ready {
                        generation,
                        request_id,
                        structure,
                    } => {
                        if app.install_structure(generation, request_id, structure, &cancelled)? {
                            structure_owner.resume = Some(ReapTicket::reaped());
                            redraw = true;
                        }
                    }
                    StructureEvent::Unavailable {
                        generation,
                        request_id,
                        reason,
                    } => {
                        if let Some(file) = &mut app.file
                            && file.build.unavailable(
                                generation,
                                request_id,
                                reason.description().into(),
                            )
                        {
                            app.structure_message = reason.description().into();
                            structure_owner.resume =
                                Some(client.cancel_request(generation, request_id));
                            redraw = true;
                        }
                    }
                }
            }
            if structure_owner.syntax_paused
                && !app.file_active()
                && !app
                    .file
                    .as_ref()
                    .is_some_and(|file| matches!(file.build.state, BuildState::Building { .. }))
                && let Some(ticket) = &structure_owner.resume
            {
                match ticket.state() {
                    ReapState::Pending => (),
                    ReapState::Reaped => {
                        if let (Some(client), Some(document)) =
                            (&syntax_owner.client, &app.syntax.document)
                        {
                            client.set_document(
                                document.original_handle(),
                                language.unwrap().to_owned(),
                                app.syntax.generation,
                            );
                        }
                        structure_owner.resume = None;
                        structure_owner.syntax_paused = false;
                    }
                    ReapState::Failed => {
                        structure_owner.resume = None;
                        app.structure_message =
                            "no se pudo recoger el motor; la fuente permanece plana".into();
                        if let Some(file) = &mut app.file {
                            file.build.state =
                                BuildState::Unavailable(app.structure_message.clone());
                        }
                        redraw = true;
                    }
                }
            }
            if !structure_owner.syntax_paused
                && !app.file_active()
                && let Some(client) = &mut syntax_owner.client
            {
                app.sync_syntax(client, language.unwrap());
                if let Some(event) = client.poll() {
                    redraw |= app.apply_syntax(event);
                }
            }
            let lost = keyboard.dropped.load(Ordering::Relaxed);
            if lost != last_dropped {
                last_dropped = lost;
                redraw = true;
            }
            if redraw {
                terminal.draw(|frame| {
                    let size = frame.area();
                    app.set_viewport(
                        usize::from(size.height.saturating_sub(3)),
                        usize::from(size.width.saturating_sub(1)),
                    );
                    if !structure_owner.syntax_paused
                        && !app.file_active()
                        && let Some(client) = &syntax_owner.client
                    {
                        app.request_syntax(client, usize::from(size.height.saturating_sub(2)));
                    }
                    render(frame, &app, title, config, lost, &cancelled);
                })?;
                redraw = false;
            }
            let event = match keyboard.rx.recv_timeout(TICK) {
                Ok(event) => event,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) if cancelled.load(Ordering::Relaxed) => {
                    return Ok(true);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(keyboard
                        .error()
                        .unwrap_or_else(|| io::Error::other("lector de teclado cerrado")));
                }
            };
            match event {
                Event::Resize(columns, rows) => {
                    app.set_viewport(
                        usize::from(rows.saturating_sub(3)),
                        usize::from(columns.saturating_sub(1)),
                    );
                    redraw = true;
                }
                Event::Key(key) => {
                    if is_cancel(&key) {
                        cancelled.store(true, Ordering::Relaxed);
                        return Ok(true);
                    }
                    if structure_toggle(&key, app.help, app.editor.is_some()) && app.file.is_some()
                    {
                        toggle_structure(
                            &mut app,
                            &mut structure_owner,
                            syntax_owner.client.as_ref(),
                            &cancelled,
                        )?;
                        redraw = true;
                        continue;
                    }
                    let area = terminal.get_frame().area();
                    if app.key(
                        key,
                        usize::from(area.height.saturating_sub(3)),
                        usize::from(area.width.saturating_sub(1)),
                        &cancelled,
                    )? {
                        return Ok(false);
                    }
                    redraw = true;
                }
                _ => {}
            }
        }
    })();
    // A keyboard cancellation discovered during preparation wins over the
    // preparation's Interrupted/error result, while I/O errors stay failures.
    let result = if cancelled.load(Ordering::Relaxed) {
        Ok(true)
    } else {
        result
    };
    structure_owner.stop();
    syntax_owner.stop();
    let restored = guard.restore();
    let _ = structure_owner.join();
    // Syntax failure is a presentation fallback, never a document exit error.
    let _ = syntax_owner.join();
    match result {
        Err(error) => Err(error),
        Ok(value) => {
            restored?;
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, data::LogicalPath};
    use clap::Parser;
    use r3viewer_wire::{Origin, Payload, Start, VERSION, Value};
    use ratatui::backend::TestBackend;
    use std::time::Instant;
    static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);
    fn config() -> PresentationConfig {
        // load() reads an explicit fixture, independent of personal config.
        let path = std::env::temp_dir().join(format!(
            "r3viewer-session-test-{}-{}.toml",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, "colour='never'\nline_numbers=false").unwrap();
        let cli = Cli::parse_from(["r3viewer", "--config", path.to_str().unwrap()]);
        let result = PresentationConfig::load(&cli, true, Arc::new(AtomicBool::new(false)));
        std::fs::remove_file(path).unwrap();
        result.unwrap()
    }
    fn model(text: &str) -> Arc<SafeTextModel> {
        Arc::new(SafeTextModel::new(text.into(), &AtomicBool::new(false)).unwrap())
    }
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn plain(
        app: &App<'_>,
        columns: u16,
        rows: u16,
        config: &PresentationConfig,
    ) -> (Terminal<TestBackend>, RenderStats) {
        let mut terminal = Terminal::new(TestBackend::new(columns, rows)).unwrap();
        let mut stats = RenderStats::default();
        terminal
            .draw(|frame| {
                stats = render(frame, app, "fixture", config, 0, &AtomicBool::new(false));
            })
            .unwrap();
        (terminal, stats)
    }
    fn contains(terminal: &Terminal<TestBackend>, needle: &str) -> bool {
        let buffer = terminal.backend().buffer();
        let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
        text.contains(needle)
    }
    fn styled_event(
        generation: u64,
        request: u64,
        viewport: Viewport,
        spans: Vec<RoleSpan>,
    ) -> SyntaxEvent {
        SyntaxEvent::Styles {
            generation,
            request,
            viewport: Some(viewport),
            spans,
        }
    }
    #[test]
    fn syntax_generation_request_and_plain_arbitration_never_revives_obsolete_styles() {
        let mut state = SyntaxView::default();
        let document = model("one\ntwo\nthree");
        assert!(state.select(Some(document.clone())));
        let generation = state.generation;
        let viewport = state.desired_viewport(0, 4096).unwrap();
        assert_eq!(viewport.line_count, 3);
        let spans = vec![RoleSpan {
            start: 0,
            end: 3,
            role: SyntaxRole::Accent,
        }];
        state.requested(viewport, 1);
        assert!(state.apply(styled_event(generation, 1, viewport, spans.clone())));
        assert_eq!(state.spans.len(), 1);
        let next_view = state.desired_viewport(1, 2).unwrap();
        state.requested(next_view, 2);
        assert!(state.spans.is_empty());
        assert!(!state.apply(styled_event(generation, 1, viewport, spans.clone())));
        assert!(!state.apply(styled_event(generation, 2, viewport, spans.clone())));
        assert!(state.apply(styled_event(generation, 2, next_view, spans.clone())));
        assert!(!state.apply(SyntaxEvent::Plain {
            generation: generation - 1,
            reason: FallbackReason::LongLine
        }));
        assert_eq!(state.spans.len(), 1);
        // Plain has no request gate: a failure from replaced viewport 1 wins.
        assert!(state.apply(SyntaxEvent::Plain {
            generation,
            reason: FallbackReason::RequestTimeout
        }));
        assert!(state.spans.is_empty());
        assert!(!state.apply(styled_event(generation, 2, next_view, spans.clone())));
        assert!(!state.apply(SyntaxEvent::Ready { generation }));
        assert!(state.desired_viewport(0, 30).is_none());
        assert!(!state.select(Some(document)));
        assert_eq!(state.plain, Some(FallbackReason::RequestTimeout));
        assert!(state.select(Some(model("new"))));
        assert_eq!(state.generation, generation + 1);
        assert!(state.plain.is_none());
        assert!(!state.apply(SyntaxEvent::Plain {
            generation,
            reason: FallbackReason::LongLine
        }));
        let huge = model(&"x\n".repeat(1000));
        assert!(state.select(Some(huge)));
        assert_eq!(state.desired_viewport(10, 1000).unwrap().line_count, 256);
        assert!(state.select(None));
        assert!(!state.apply(styled_event(state.generation, 2, next_view, spans)));
    }
    #[test]
    fn syntax_applies_only_to_string_panes_and_same_leaf_search_preserves_generation() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let typed = Arc::new(TypedInput {
            start: Start {
                version: VERSION,
                cwd: "irrelevant.rs".into(),
                origin: Origin::SingleValue,
            },
            payload: Payload::SingleValue(Value::Record(vec![
                ("a".into(), Value::String("x x".into())),
                ("b".into(), Value::String("second".into())),
                ("number".into(), Value::Int(7)),
            ])),
        });
        let mut app = App::new(
            model(""),
            Some(typed.clone()),
            Some(&typed),
            &config,
            &cancel,
        )
        .unwrap();
        assert!(app.syntax_document().is_none());
        app.key(key(KeyCode::Enter), 8, 40, &cancel).unwrap();
        assert!(app.pane.as_ref().unwrap().key);
        assert!(app.syntax_document().is_none());
        app.key(key(KeyCode::Backspace), 8, 40, &cancel).unwrap();
        let first = SearchHit {
            path: Some(LogicalPath(vec![PathMember::Key("a".into())])),
            key: false,
            visit_index: 1,
            range: 0..1,
        };
        app.apply_search(SearchOutcome::Found(first.clone()), &cancel)
            .unwrap();
        let document = app.syntax_document().unwrap();
        assert!(app.syntax.select(Some(document.clone())));
        let generation = app.syntax.generation;
        app.apply_syntax(SyntaxEvent::Plain {
            generation,
            reason: FallbackReason::LongLine,
        });
        let mut second = first;
        second.range = 2..3;
        app.apply_search(SearchOutcome::Found(second), &cancel)
            .unwrap();
        assert!(Arc::ptr_eq(&document, &app.syntax_document().unwrap()));
        assert!(!app.syntax.select(app.syntax_document()));
        assert_eq!(app.syntax.generation, generation);
        assert!(app.syntax.plain.is_some());
        app.key(key(KeyCode::Backspace), 8, 40, &cancel).unwrap();
        assert!(app.syntax.select(app.syntax_document()));
        app.navigator.as_mut().unwrap().current_mut().cursor.row = 2;
        app.navigator.as_mut().unwrap().current_mut().cursor.column = 1;
        app.key(key(KeyCode::Enter), 8, 40, &cancel).unwrap();
        assert_eq!(app.pane.as_ref().unwrap().model.original(), "int(7)");
        assert!(app.syntax_document().is_none());
        let plain = App::new(model("fn main() {}"), None, None, &config, &cancel).unwrap();
        assert!(plain.syntax_document().is_some());
    }
    #[test]
    fn explicit_string_stream_is_textual_but_unknown_and_binary_remain_descriptors() {
        let config = config();
        let cancel = AtomicBool::new(false);
        for kind in [
            ByteStreamKind::String,
            ByteStreamKind::Unknown,
            ByteStreamKind::Binary,
        ] {
            let typed = Arc::new(TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "irrelevant.rs".into(),
                    origin: Origin::ByteStream(kind),
                },
                payload: Payload::ByteStream {
                    kind,
                    bytes: b"fn main() {}\n".to_vec(),
                },
            });
            assert_eq!(
                crate::syntax_selection::textual_input(Some(&typed)),
                kind == ByteStreamKind::String
            );
            let mut app = App::new(
                model(""),
                Some(typed.clone()),
                Some(&typed),
                &config,
                &cancel,
            )
            .unwrap();
            if kind == ByteStreamKind::String {
                let original = app.syntax_document().unwrap();
                assert_eq!(original.original(), "fn main() {}\n");
                app.syntax.select(Some(original.clone()));
                let generation = app.syntax.generation;
                app.apply_syntax(SyntaxEvent::Plain {
                    generation,
                    reason: FallbackReason::LongLine,
                });
                app.apply_search(
                    SearchOutcome::Found(SearchHit {
                        path: Some(LogicalPath(vec![])),
                        key: false,
                        visit_index: 0,
                        range: 0..2,
                    }),
                    &cancel,
                )
                .unwrap();
                assert!(Arc::ptr_eq(&original, &app.syntax_document().unwrap()));
                assert!(!app.syntax.select(app.syntax_document()));
                assert!(app.syntax.plain.is_some());
            } else {
                assert!(app.syntax_document().is_none());
            }
        }
    }
    #[test]
    fn syntax_roles_follow_first_original_scalar_and_search_composes_on_clusters() {
        let mut config = config();
        config.coloured = true;
        let cancel = AtomicBool::new(false);
        let document = model("e\u{301}👩‍💻\u{009b}␛\x1bTOKEN\r\n\t界");
        let spans = vec![
            RoleSpan {
                start: 0,
                end: 1,
                role: SyntaxRole::Accent,
            },
            RoleSpan {
                start: 1,
                end: 3,
                role: SyntaxRole::Success,
            },
            RoleSpan {
                start: 3,
                end: 7,
                role: SyntaxRole::Secondary,
            },
            RoleSpan {
                start: 7,
                end: 10,
                role: SyntaxRole::Accent,
            },
            RoleSpan {
                start: 10,
                end: 14,
                role: SyntaxRole::Success,
            },
            RoleSpan {
                start: 14,
                end: 16,
                role: SyntaxRole::Process,
            },
            RoleSpan {
                start: 16,
                end: 19,
                role: SyntaxRole::Heading,
            },
            RoleSpan {
                start: 19,
                end: 20,
                role: SyntaxRole::Secondary,
            },
            RoleSpan {
                start: 20,
                end: 25,
                role: SyntaxRole::Success,
            },
        ];
        let mut app = App::new(document.clone(), None, None, &config, &cancel).unwrap();
        app.syntax.select(Some(document.clone()));
        let viewport = Viewport {
            first_line: 0,
            line_count: 2,
        };
        app.syntax.requested(viewport, 9);
        assert!(app.apply_syntax(styled_event(
            app.syntax.generation,
            9,
            viewport,
            spans.clone()
        )));
        app.pane.as_mut().unwrap().hit = Some(1..3);
        let (terminal, _) = plain(&app, 40, 6, &config);
        let cells = terminal.backend().buffer();
        assert_eq!(cells[(0, 1)].symbol(), "e\u{301}");
        assert_eq!(cells[(0, 1)].fg, config.style(Role::Accent).fg.unwrap());
        assert!(
            cells[(0, 1)]
                .modifier
                .contains(Modifier::REVERSED | Modifier::BOLD)
        );
        assert_eq!(cells[(1, 1)].symbol(), "👩‍💻");
        assert_eq!(cells[(1, 1)].fg, config.style(Role::Secondary).fg.unwrap());
        for x in 3..11 {
            assert_eq!(cells[(x, 1)].fg, config.style(Role::Process).fg.unwrap());
        }
        assert_eq!(cells[(11, 1)].fg, config.style(Role::Heading).fg.unwrap());
        assert_eq!(cells[(12, 1)].fg, config.style(Role::Secondary).fg.unwrap());
        assert_eq!(cells[(13, 1)].fg, config.style(Role::Success).fg.unwrap());
        // Cropping inside a C1 placeholder still maps every visible scalar to C1.
        app.pane.as_mut().unwrap().position.left = 6;
        let (cropped, _) = plain(&app, 7, 4, &config);
        for x in 0..5 {
            assert_eq!(
                cropped.backend().buffer()[(x, 1)].fg,
                config.style(Role::Process).fg.unwrap()
            );
        }
        let generation = app.syntax.generation;
        assert!(app.apply_syntax(SyntaxEvent::Plain {
            generation,
            reason: FallbackReason::LongLine
        }));
        app.pane.as_mut().unwrap().position = Position::default();
        let (fallback, _) = plain(&app, 40, 6, &config);
        let first = &fallback.backend().buffer()[(0, 1)];
        assert_eq!(first.fg, config.style(Role::Value).fg.unwrap());
        assert!(first.modifier.contains(Modifier::REVERSED | Modifier::BOLD));
        // A role only on a later combining/ZWJ scalar must not color its cluster.
        let late = [spans[1], spans[3], spans[4]];
        let line = text_line(
            &document,
            0,
            0..20,
            TextDecoration {
                hit: None,
                syntax: &late,
            },
            &config,
            &cancel,
            &mut RenderStats::default(),
        );
        assert_eq!(line.spans[0].style.fg, config.style(Role::Value).fg);
        assert_eq!(line.spans[1].style.fg, config.style(Role::Value).fg);
    }
    #[test]
    fn file_source_domains_keep_queries_hits_cursor_and_complete_raw_jump_independent() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let structure = crate::data::file_fixture::sample();
        let source = model(structure.source());
        let mut app = App::new(source.clone(), None, None, &config, &cancel).unwrap();
        app.file = Some(FileWorkspace::new(1, structure.source_handle()));
        app.query = "\\u754c".into();
        app.last_hit = Some(SearchHit {
            path: None,
            key: false,
            visit_index: 0,
            range: 7..13,
        });
        app.pane.as_mut().unwrap().position.top = 1;
        let request = app.file.as_mut().unwrap().build.begin().unwrap();
        // Same bytes and generation are insufficient: another snapshot's arena
        // cannot retire the current request or create raw navigation offsets.
        assert!(
            !app.install_structure(1, request, crate::data::file_fixture::sample(), &cancel)
                .unwrap()
        );
        assert!(app.file.as_ref().unwrap().build.current(1, request));
        assert!(
            !app.install_structure(0, request, structure.clone(), &cancel)
                .unwrap()
        );
        assert!(
            app.install_structure(1, request, structure.clone(), &cancel)
                .unwrap()
        );
        assert!(app.file_active());
        assert!(app.query.is_empty());
        app.query = "界\x1b".into();
        let SearchOutcome::FileFound(hit) =
            find_structure(&structure, &app.query, None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        app.apply_search(SearchOutcome::FileFound(hit.clone()), &cancel)
            .unwrap();
        assert_eq!(app.pane.as_ref().unwrap().model.original(), "界\x1b");
        assert!(app.syntax_document().is_none());
        app.file_key(KeyCode::Char('o'), 8, 40, &cancel).unwrap();
        assert!(!app.file_active());
        assert_eq!(app.query, "\\u754c");
        assert_eq!(app.last_hit.as_ref().unwrap().range, 7..13);
        let raw = structure
            .root()
            .entry(0)
            .unwrap()
            .value()
            .source_location()
            .unwrap()
            .range();
        assert_eq!(
            app.pane.as_ref().unwrap().hit,
            Some(source.visible_range(raw))
        );
        app.swap_file_domain(&cancel).unwrap();
        assert!(app.file_active());
        assert_eq!(app.query, "界\x1b");
        assert_eq!(app.file_hit.as_ref().unwrap().range, hit.range);
        assert_eq!(
            app.file
                .as_ref()
                .unwrap()
                .navigator
                .as_ref()
                .unwrap()
                .current()
                .path,
            hit.path
        );
        app.key(key(KeyCode::Backspace), 8, 40, &cancel).unwrap();
        assert!(app.pane.is_none());
        let nav = app.file.as_mut().unwrap().navigator.as_mut().unwrap();
        nav.current_mut().cursor.row = 1;
        nav.current_mut().cursor.column = 1;
        nav.enter(1, 1, &cancel).unwrap();
        // Inline array has a complete location; its parent is an implicit table.
        nav.back();
        nav.current_mut().cursor.row = 1;
        app.refresh_pane(&cancel).unwrap();
        assert!(app.file_key(KeyCode::Char('o'), 8, 40, &cancel).unwrap());
    }
    #[test]
    fn ready_during_query_edit_stays_in_source_and_missing_raw_span_is_diagnostic() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let structure = crate::data::file_fixture::sample();
        let mut app = App::new(model(structure.source()), None, None, &config, &cancel).unwrap();
        app.file = Some(FileWorkspace::new(1, structure.source_handle()));
        app.query = "source-query".into();
        app.key(key(KeyCode::Char('/')), 8, 40, &cancel).unwrap();
        let request = app.file.as_mut().unwrap().build.begin().unwrap();
        assert!(
            app.install_structure(1, request, structure.clone(), &cancel)
                .unwrap()
        );
        assert!(!app.file_active());
        assert!(app.editor.is_some());
        app.key(key(KeyCode::Esc), 8, 40, &cancel).unwrap();
        app.swap_file_domain(&cancel).unwrap();
        assert!(app.file_active());
        // An implicit empty table has no fabricated source location.
        let mut builder =
            crate::structure_model::StructureBuilder::new(structure.source_handle(), 1).unwrap();
        let root = builder
            .push_node(crate::structure_model::NodeSpec {
                kind: crate::structure_model::StructureKind::Mapping,
                text: None,
                location: None,
            })
            .unwrap();
        builder.set_entries(root, []).unwrap();
        let empty = Arc::new(builder.finish(root, 0, 0, &cancel).unwrap());
        app.file.as_mut().unwrap().navigator = Some(FileNavigator::new(empty));
        app.refresh_pane(&cancel).unwrap();
        assert!(app.file_key(KeyCode::Char('o'), 8, 40, &cancel).unwrap());
        assert!(app.file_active());
        assert!(app.message.contains("span original"));
        assert_eq!(app.file.as_ref().unwrap().inactive.query, "source-query");
        assert!(
            !app.install_structure(1, request, structure, &cancel)
                .unwrap()
        );
    }
    #[test]
    fn file_hundred_thousand_rows_render_only_viewport_and_keep_tiny_cell_selection() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let structure = crate::data::file_fixture::sequence(100_000);
        let mut app = App::new(model("1"), None, None, &config, &cancel).unwrap();
        app.file = Some(FileWorkspace::new(1, structure.source_handle()));
        let request = app.file.as_mut().unwrap().build.begin().unwrap();
        assert!(
            app.install_structure(1, request, structure, &cancel)
                .unwrap()
        );
        let cursor = &mut app
            .file
            .as_mut()
            .unwrap()
            .navigator
            .as_mut()
            .unwrap()
            .current_mut()
            .cursor;
        cursor.row = 99_980;
        cursor.top = 99_980;
        cursor.column = 1;
        let (_, stats) = plain(&app, 80, 12, &config);
        assert_eq!(stats.rows, 9);
        assert_eq!(stats.cells, 18);
        let (tiny, stats) = plain(&app, 2, 4, &config);
        assert_eq!(stats.rows, 1);
        assert_eq!(stats.cells, 1);
        assert!(
            tiny.backend().buffer()[(0, 2)]
                .modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(
            app.file
                .as_ref()
                .unwrap()
                .navigator
                .as_ref()
                .unwrap()
                .current()
                .cursor
                .row,
            99_980
        );
    }
    #[test]
    fn structure_build_cancel_retries_explicitly_and_errors_are_latched_per_snapshot() {
        let mut state = BuildSelection::new(9);
        let first = state.begin().unwrap();
        assert!(state.begin().is_none());
        assert_eq!(state.cancel_build(), Some(first));
        assert!(!state.ready(9, first));
        let retry = state.begin().unwrap();
        assert!(retry > first);
        assert!(!state.unavailable(8, retry, "old generation".into()));
        assert!(!state.unavailable(9, first, "old request".into()));
        assert!(state.unavailable(9, retry, "invalid TOML".into()));
        assert!(state.begin().is_none());
        assert!(state.cancel_build().is_none());
        assert!(!state.ready(9, retry));
        assert_eq!(state.state, BuildState::Unavailable("invalid TOML".into()));
    }
    #[test]
    fn structure_tab_only_toggles_on_unmodified_press_outside_editor_and_help() {
        for code in [KeyCode::Tab, KeyCode::BackTab] {
            let press = KeyEvent::new(code, KeyModifiers::SHIFT);
            assert!(structure_toggle(&press, false, false));
            assert!(!structure_toggle(&press, true, false));
            assert!(!structure_toggle(&press, false, true));
            for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
                assert!(!structure_toggle(
                    &KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind),
                    false,
                    false
                ));
            }
            assert!(!structure_toggle(
                &KeyEvent::new(code, KeyModifiers::CONTROL | KeyModifiers::ALT),
                false,
                false
            ));
        }
    }
    #[test]
    fn unicode_highlights_use_safe_bytes_whole_graphemes_and_resize_anchor() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let source = model("\x1bXfoo\r\ne\u{301}界👩‍💻\tend");
        let mut app = App::new(source, None, None, &config, &cancel).unwrap();
        app.pane.as_mut().unwrap().hit = Some(4..7);
        let (terminal, stats) = plain(&app, 24, 6, &config);
        assert!(contains(&terminal, "␛Xfoo"));
        assert_eq!(stats.rows, 2);
        let buffer = terminal.backend().buffer();
        assert!(buffer[(2, 1)].modifier.contains(Modifier::REVERSED));
        assert!(!buffer[(1, 1)].modifier.contains(Modifier::REVERSED));
        let pane = app.pane.as_mut().unwrap();
        let combining = pane.model.visible().find('\u{301}').unwrap();
        pane.hit = Some(combining..combining + 2);
        pane.position.top = 1;
        pane.position.left = offset_cell(&pane.model, combining, &cancel).unwrap();
        assert_eq!(pane.position.left, 0);
        let (terminal, _) = plain(&app, 12, 4, &config);
        assert_eq!(terminal.backend().buffer()[(0, 1)].symbol(), "e\u{301}");
        assert!(
            terminal.backend().buffer()[(0, 1)]
                .modifier
                .contains(Modifier::REVERSED)
        );
        let pane = app.pane.as_mut().unwrap();
        pane.position.left = 2;
        let anchor = pane.position;
        let (_, _) = plain(&app, 6, 3, &config);
        assert_eq!(app.pane.as_ref().unwrap().position.top, anchor.top);
        assert_eq!(app.pane.as_ref().unwrap().position.left, anchor.left);
    }
    #[test]
    fn real_hundred_thousand_row_render_reads_only_viewport_and_selected_cell() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let typed = Arc::new(TypedInput {
            start: Start {
                version: VERSION,
                cwd: "never-reopen".into(),
                origin: Origin::ListStream,
            },
            payload: Payload::ListStream(
                (0..100_000)
                    .map(|n| {
                        Value::Record(vec![
                            ("n".into(), Value::Int(n)),
                            ("value".into(), Value::String("🦊".into())),
                        ])
                    })
                    .collect(),
            ),
        });
        let before = Instant::now();
        let mut app = App::new(
            model(""),
            Some(typed.clone()),
            Some(&typed),
            &config,
            &cancel,
        )
        .unwrap();
        let preparation = before.elapsed();
        let view = app.navigator.as_mut().unwrap().current_mut();
        assert_eq!(view.preparation.rows_scanned, 100_000);
        assert_eq!(view.preparation.fields_scanned, 200_000);
        view.cursor.row = 99_980;
        view.cursor.top = 99_980;
        view.cursor.column = 1;
        let (terminal, stats) = plain(&app, 80, 12, &config);
        assert_eq!(stats.rows, 9);
        assert_eq!(stats.cells, 18);
        assert!(
            terminal.backend().buffer()[(24, 2)]
                .modifier
                .contains(Modifier::REVERSED)
        );
        assert!(
            !terminal.backend().buffer()[(0, 2)]
                .modifier
                .contains(Modifier::REVERSED)
        );
        let (_, small) = plain(&app, 30, 5, &config);
        assert_eq!(small.rows, 2);
        assert_eq!(small.cells, 2);
        assert_eq!(
            app.navigator
                .as_ref()
                .unwrap()
                .current()
                .preparation
                .rows_scanned,
            100_000
        );
        eprintln!(
            "100k initial preparation {preparation:?}; viewport visits {} rows/{} cells",
            stats.rows, stats.cells
        );
    }
    #[test]
    fn all_record_and_table_keys_are_fully_inspectable_and_parent_cursor_restored() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let long_key = format!("{}\x1bend-key", "a".repeat(500));
        let typed = Arc::new(TypedInput {
            start: Start {
                version: VERSION,
                cwd: "x".into(),
                origin: Origin::SingleValue,
            },
            payload: Payload::SingleValue(Value::List(vec![Value::Record(vec![(
                long_key.clone(),
                Value::String("complete-value".into()),
            )])])),
        });
        let mut app = App::new(
            model(""),
            Some(typed.clone()),
            Some(&typed),
            &config,
            &cancel,
        )
        .unwrap();
        app.key(key(KeyCode::Char('K')), 8, 40, &cancel).unwrap();
        let pane = app.pane.as_ref().unwrap();
        assert!(pane.key);
        assert_eq!(pane.model.original(), long_key);
        assert!(pane.model.visible().ends_with("␛end-key"));
        app.key(key(KeyCode::Backspace), 8, 40, &cancel).unwrap();
        assert!(app.pane.is_none());
        app.key(key(KeyCode::Enter), 8, 40, &cancel).unwrap();
        assert_eq!(
            app.pane.as_ref().unwrap().model.original(),
            "complete-value"
        );
        app.key(key(KeyCode::Backspace), 8, 40, &cancel).unwrap();
        assert_eq!(app.navigator.as_ref().unwrap().current().cursor.row, 0);
        assert_eq!(app.navigator.as_ref().unwrap().current().cursor.column, 0);
        let key_hit = SearchHit {
            path: Some(LogicalPath(vec![
                PathMember::Index(0),
                PathMember::Key(long_key),
            ])),
            key: true,
            visit_index: 1,
            range: 500..503,
        };
        app.apply_search(SearchOutcome::Found(key_hit), &cancel)
            .unwrap();
        assert!(app.pane.as_ref().unwrap().key);
        assert_eq!(app.navigator.as_ref().unwrap().current().cursor.column, 0);
        let (terminal, _) = plain(&app, 30, 5, &config);
        assert!(
            terminal.backend().buffer()[(0, 1)]
                .modifier
                .contains(Modifier::REVERSED)
        );
    }
    #[test]
    fn altgr_printable_editor_input_is_literal_and_release_is_not_duplicated() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let mut app = App::new(model("á@{cC␛"), None, None, &config, &cancel).unwrap();
        app.key(key(KeyCode::Char('/')), 4, 30, &cancel).unwrap();
        let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
        for ch in ['á', '@', '{', 'c', 'C'] {
            let press = KeyEvent::new(KeyCode::Char(ch), altgr);
            assert!(!is_cancel(&press));
            assert!(!app.key(press, 4, 30, &cancel).unwrap());
            let release = KeyEvent::new_with_kind(KeyCode::Char(ch), altgr, KeyEventKind::Release);
            app.key(release, 4, 30, &cancel).unwrap();
        }
        // The Windows backend normalizes completed Alt-code text to Press.
        app.key(key(KeyCode::Char('␛')), 4, 30, &cancel).unwrap();
        app.key(
            KeyEvent::new_with_kind(
                KeyCode::Char('␛'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            4,
            30,
            &cancel,
        )
        .unwrap();
        for modifiers in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
            app.key(KeyEvent::new(KeyCode::Char('x'), modifiers), 4, 30, &cancel)
                .unwrap();
        }
        for ch in ['\u{3}', '\u{1b}', '\n', '\t', '\u{7f}'] {
            app.key(key(KeyCode::Char(ch)), 4, 30, &cancel).unwrap();
        }
        assert_eq!(app.editor.as_deref(), Some("á@{cC␛"));
        assert!(!cancel.load(Ordering::Relaxed));
        app.key(key(KeyCode::Enter), 4, 30, &cancel).unwrap();
        assert_eq!(app.query, "á@{cC␛");
        assert!(app.editor.is_none());
    }
    #[test]
    fn modified_character_events_do_not_execute_viewer_commands() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let mut app = App::new(model("one\ntwo\nthree"), None, None, &config, &cancel).unwrap();
        for modifiers in [
            KeyModifiers::ALT,
            KeyModifiers::CONTROL | KeyModifiers::ALT,
            KeyModifiers::CONTROL,
        ] {
            for ch in ['q', '/', '?', 'n', 'N', 'L', 'j', 'l', 'K', 'r'] {
                assert!(
                    !app.key(KeyEvent::new(KeyCode::Char(ch), modifiers), 2, 30, &cancel)
                        .unwrap()
                );
                assert!(app.editor.is_none());
                assert!(!app.help);
                assert!(!app.searching);
                assert!(!app.line_numbers);
                assert_eq!(app.pane.as_ref().unwrap().position, Position::default());
            }
        }
        // Unmodified commands and Shift+N still dispatch normally.
        app.query = "one".into();
        app.key(
            KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT),
            2,
            30,
            &cancel,
        )
        .unwrap();
        assert!(app.searching);
        assert!(app.key(key(KeyCode::Char('q')), 2, 30, &cancel).unwrap());
    }
    #[test]
    fn cancellation_distinguishes_altgr_c_and_raw_etx_even_with_full_queue() {
        let (tx, _rx) = mpsc::sync_channel(1);
        let cancel = AtomicBool::new(false);
        let dropped = AtomicUsize::new(0);
        assert!(enqueue(
            &tx,
            Event::Key(key(KeyCode::Char('x'))),
            &cancel,
            &dropped
        ));
        for ch in ['c', 'C'] {
            let altgr = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL | KeyModifiers::ALT);
            assert!(!is_cancel(&altgr));
            assert!(enqueue(&tx, Event::Key(altgr), &cancel, &dropped));
            assert!(!cancel.load(Ordering::Relaxed));
        }
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
            KeyEventKind::Release,
        );
        assert!(!is_cancel(&release));
        assert!(enqueue(&tx, Event::Key(release), &cancel, &dropped));
        assert!(!cancel.load(Ordering::Relaxed));
        for modifiers in [
            KeyModifiers::NONE,
            KeyModifiers::ALT,
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ] {
            let raw = KeyEvent::new(KeyCode::Char('\u{3}'), modifiers);
            assert!(is_cancel(&raw));
            assert!(enqueue(&tx, Event::Key(raw), &cancel, &dropped));
            assert!(cancel.swap(false, Ordering::Relaxed));
        }
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(enqueue(&tx, Event::Key(ctrl_c), &cancel, &dropped));
        assert!(cancel.load(Ordering::Relaxed));
        assert_eq!(dropped.load(Ordering::Relaxed), 7);
    }
    #[test]
    fn search_editor_cancel_limit_and_n_next_are_literal() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let mut app = App::new(model("aba aba"), None, None, &config, &cancel).unwrap();
        app.key(key(KeyCode::Char('/')), 4, 30, &cancel).unwrap();
        app.key(key(KeyCode::Char('q')), 4, 30, &cancel).unwrap();
        assert_eq!(app.editor.as_deref(), Some("q"));
        assert!(!app.key(key(KeyCode::Esc), 4, 30, &cancel).unwrap());
        assert!(app.editor.is_none());
        assert!(app.query.is_empty());
        app.editor = Some("x".repeat(QUERY_LIMIT));
        app.key(key(KeyCode::Char('🦊')), 4, 30, &cancel).unwrap();
        assert_eq!(app.editor.as_ref().unwrap().len(), QUERY_LIMIT);
        app.editor = Some("aba".into());
        app.key(key(KeyCode::Enter), 4, 30, &cancel).unwrap();
        let start = Instant::now();
        loop {
            if let Some(outcome) = app.worker.poll() {
                app.apply_search(outcome, &cancel).unwrap();
                break;
            }
            assert!(start.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(app.last_hit.as_ref().unwrap().range, 0..3);
        app.key(key(KeyCode::Char('n')), 4, 30, &cancel).unwrap();
        loop {
            if let Some(outcome) = app.worker.poll() {
                app.apply_search(outcome, &cancel).unwrap();
                break;
            }
            assert!(start.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(app.last_hit.as_ref().unwrap().range, 4..7);
    }
    #[test]
    fn empty_records_enter_rows_and_r_work_without_fictitious_keys() {
        let config = config();
        let cancel = AtomicBool::new(false);
        for rows in [
            vec![Value::Record(vec![]); 3],
            vec![
                Value::Record(vec![]),
                Value::Record(vec![("x".into(), Value::Nothing)]),
            ],
        ] {
            let typed = Arc::new(TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "x".into(),
                    origin: Origin::ListStream,
                },
                payload: Payload::ListStream(rows),
            });
            let mut app = App::new(
                model(""),
                Some(typed.clone()),
                Some(&typed),
                &config,
                &cancel,
            )
            .unwrap();
            app.key(key(KeyCode::Char('/')), 5, 40, &cancel).unwrap();
            app.key(key(KeyCode::Char('r')), 5, 40, &cancel).unwrap();
            assert_eq!(app.editor.as_deref(), Some("r"));
            app.key(key(KeyCode::Esc), 5, 40, &cancel).unwrap();
            app.key(key(KeyCode::Char('r')), 5, 40, &cancel).unwrap();
            assert_eq!(app.navigator.as_ref().unwrap().current().row_count(), 0);
            assert_eq!(
                app.navigator.as_ref().unwrap().current().path,
                LogicalPath(vec![PathMember::Index(0)])
            );
            let (terminal, _) = plain(&app, 45, 5, &config);
            assert!(contains(&terminal, "record{0 fields}"));
            app.key(key(KeyCode::Backspace), 5, 40, &cancel).unwrap();
            assert!(app.navigator.as_ref().unwrap().current().is_table());
        }
    }
    #[test]
    fn keyboard_thread_cancels_without_consumption_and_drop_reaps_it() {
        let cancel = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let keyboard = KeyboardReader::spawn(cancel.clone(), move || {
            let index = counter.fetch_add(1, Ordering::Relaxed);
            Ok(Some(Event::Key(if index < EVENT_CAPACITY * 2 {
                key(KeyCode::Char('x'))
            } else {
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
            })))
        })
        .unwrap();
        let before = Instant::now();
        while !cancel.load(Ordering::Relaxed) {
            assert!(before.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(1));
        }
        assert!(keyboard.dropped.load(Ordering::Relaxed) > 0);
        drop(keyboard);
        let after = calls.load(Ordering::Relaxed);
        thread::sleep(Duration::from_millis(10));
        assert_eq!(calls.load(Ordering::Relaxed), after);
    }
    #[test]
    fn tiny_viewports_keep_selected_cells_and_never_access_outside_buffer() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let typed = Arc::new(TypedInput {
            start: Start {
                version: VERSION,
                cwd: "x".into(),
                origin: Origin::SingleValue,
            },
            payload: Payload::SingleValue(Value::Record(vec![(
                "key".into(),
                Value::String("value".into()),
            )])),
        });
        let mut app = App::new(
            model(""),
            Some(typed.clone()),
            Some(&typed),
            &config,
            &cancel,
        )
        .unwrap();
        app.navigator.as_mut().unwrap().current_mut().cursor.column = 1;
        for (columns, rows) in [(1, 1), (2, 2), (2, 4), (3, 3)] {
            let _ = plain(&app, columns, rows, &config);
        }
        let (terminal, _) = plain(&app, 2, 5, &config);
        assert!(
            terminal.backend().buffer()[(0, 2)]
                .modifier
                .contains(Modifier::REVERSED)
        );
    }
    #[test]
    fn pending_search_results_cannot_move_views_after_navigation() {
        use std::sync::Barrier;
        let config = config();
        let cancel = AtomicBool::new(false);
        for code in [
            KeyCode::Enter,
            KeyCode::Char('K'),
            KeyCode::Char('r'),
            KeyCode::Backspace,
            KeyCode::Right,
            KeyCode::Char('?'),
        ] {
            let typed = Arc::new(TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "x".into(),
                    origin: Origin::ListStream,
                },
                payload: Payload::ListStream(vec![
                    Value::Record(vec![("a".into(), Value::String("old".into()))]),
                    Value::Record(vec![("b".into(), Value::String("new".into()))]),
                ]),
            });
            let mut app = App::new(
                model(""),
                Some(typed.clone()),
                Some(&typed),
                &config,
                &cancel,
            )
            .unwrap();
            app.navigator.as_mut().unwrap().current_mut().cursor.row = 1;
            app.navigator.as_mut().unwrap().current_mut().cursor.column = 1;
            if matches!(code, KeyCode::Backspace | KeyCode::Right) {
                app.key(key(KeyCode::Enter), 5, 80, &cancel).unwrap();
            }
            let started = Arc::new(Barrier::new(2));
            let release = Arc::new(Barrier::new(2));
            let finished = Arc::new(AtomicBool::new(false));
            let worker_started = started.clone();
            let worker_release = release.clone();
            let worker_finished = finished.clone();
            app.last_hit = Some(SearchHit {
                path: None,
                key: false,
                visit_index: 99,
                range: 0..1,
            });
            app.worker.submit(move |_| {
                worker_started.wait();
                worker_release.wait();
                worker_finished.store(true, Ordering::Relaxed);
                // Deliberately ignores the token: cancellation must invalidate
                // delivery even if completion races with the navigation.
                SearchOutcome::Found(SearchHit {
                    path: Some(LogicalPath(vec![
                        PathMember::Index(0),
                        PathMember::Key("a".into()),
                    ])),
                    key: false,
                    visit_index: 1,
                    range: 0..3,
                })
            });
            app.searching = true;
            started.wait();
            app.key(key(code), 5, 80, &cancel).unwrap();
            assert!(!app.searching, "{code:?}");
            let path = app.navigator.as_ref().unwrap().current().path.clone();
            let left = app.pane.as_ref().map(|p| p.position.left);
            release.wait();
            let before = Instant::now();
            while !finished.load(Ordering::Relaxed) {
                assert!(before.elapsed() < Duration::from_secs(2));
                thread::yield_now();
            }
            assert!(app.worker.poll().is_none(), "stale result from {code:?}");
            assert_eq!(app.navigator.as_ref().unwrap().current().path, path);
            assert_eq!(app.pane.as_ref().map(|p| p.position.left), left);
            assert_eq!(app.last_hit.as_ref().unwrap().visit_index, 99);
        }
    }
    #[test]
    fn horizontal_scroll_uses_tab_cells_and_sparse_viewport_checkpoints() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let raw = format!("{}end", "\t".repeat(20));
        let mut app = App::new(model(&raw), None, None, &config, &cancel).unwrap();
        assert_eq!(app.source.visible().len(), 23);
        assert_eq!(app.source.max_width(), 163);
        app.apply_search(
            SearchOutcome::Found(SearchHit {
                path: None,
                key: false,
                visit_index: 0,
                range: 20..23,
            }),
            &cancel,
        )
        .unwrap();
        assert_eq!(app.pane.as_ref().unwrap().position.left, 160);
        app.key(key(KeyCode::Right), 4, 8, &cancel).unwrap();
        assert_eq!(app.pane.as_ref().unwrap().position.left, 161);
        app.key(key(KeyCode::Left), 4, 8, &cancel).unwrap();
        assert_eq!(app.pane.as_ref().unwrap().position.left, 160);
        let (terminal, _) = plain(&app, 9, 4, &config);
        assert!(contains(&terminal, "end"));
        let long = format!("{}needle", "x".repeat(100_000));
        let mut app = App::new(model(&long), None, None, &config, &cancel).unwrap();
        app.apply_search(
            SearchOutcome::Found(SearchHit {
                path: None,
                key: false,
                visit_index: 0,
                range: 100_000..100_006,
            }),
            &cancel,
        )
        .unwrap();
        let (terminal, stats) = plain(&app, 13, 4, &config);
        assert!(contains(&terminal, "needle"));
        assert!(
            stats.graphemes <= 4096 + 12,
            "visited {} graphemes",
            stats.graphemes
        );
    }
    #[test]
    fn shrinking_and_back_after_distant_hit_reveal_selection_without_resetting_anchor() {
        let config = config();
        let cancel = AtomicBool::new(false);
        let typed = Arc::new(TypedInput {
            start: Start {
                version: VERSION,
                cwd: "x".into(),
                origin: Origin::ListStream,
            },
            payload: Payload::ListStream(
                (0..10)
                    .map(|n| {
                        Value::Record(vec![
                            ("a".into(), Value::Int(n)),
                            ("b".into(), Value::Nothing),
                            ("c".into(), Value::Bool(true)),
                        ])
                    })
                    .collect(),
            ),
        });
        let mut app = App::new(
            model(""),
            Some(typed.clone()),
            Some(&typed),
            &config,
            &cancel,
        )
        .unwrap();
        app.set_viewport(20, 110);
        let view = app.navigator.as_mut().unwrap().current_mut();
        view.cursor.row = 8;
        view.cursor.column = 2;
        app.set_viewport(1, 24);
        let view = app.navigator.as_ref().unwrap().current();
        assert_eq!(
            (
                view.cursor.row,
                view.cursor.column,
                view.cursor.top,
                view.cursor.left
            ),
            (8, 2, 8, 2)
        );
        let path = view.selected_path();
        app.set_viewport(20, 110);
        assert_eq!(app.navigator.as_ref().unwrap().current().cursor.top, 8);
        assert_eq!(app.navigator.as_ref().unwrap().current().cursor.left, 2);
        assert_eq!(
            app.navigator.as_ref().unwrap().current().selected_path(),
            path
        );
        let typed = Arc::new(TypedInput {
            start: Start {
                version: VERSION,
                cwd: "x".into(),
                origin: Origin::ListStream,
            },
            payload: Payload::ListStream(
                (0..100_000)
                    .map(|n| Value::Record(vec![("n".into(), Value::Int(n))]))
                    .collect(),
            ),
        });
        let mut app = App::new(
            model(""),
            Some(typed.clone()),
            Some(&typed),
            &config,
            &cancel,
        )
        .unwrap();
        app.set_viewport(5, 79);
        let path = LogicalPath(vec![PathMember::Index(99_985), PathMember::Key("n".into())]);
        app.apply_search(
            SearchOutcome::Found(SearchHit {
                path: Some(path.clone()),
                key: false,
                visit_index: 0,
                range: 0..3,
            }),
            &cancel,
        )
        .unwrap();
        app.key(key(KeyCode::Backspace), 5, 79, &cancel).unwrap();
        app.key(key(KeyCode::Backspace), 5, 79, &cancel).unwrap();
        let view = app.navigator.as_ref().unwrap().current();
        assert_eq!(view.cursor.row, 99_985);
        assert_eq!(view.cursor.top, 99_981);
        assert_eq!(view.selected_path(), Some(path));
        let (terminal, stats) = plain(&app, 80, 8, &config);
        assert_eq!(stats.rows, 5);
        assert!(
            terminal.backend().buffer()[(0, 6)]
                .modifier
                .contains(Modifier::REVERSED)
        );
    }
    #[test]
    fn full_keyboard_queue_cannot_swallow_ctrl_c_and_overflow_is_visible() {
        let (tx, _rx) = mpsc::sync_channel(1);
        let cancel = AtomicBool::new(false);
        let lost = AtomicUsize::new(0);
        assert!(enqueue(
            &tx,
            Event::Key(key(KeyCode::Char('x'))),
            &cancel,
            &lost
        ));
        assert!(enqueue(
            &tx,
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            &cancel,
            &lost
        ));
        assert!(cancel.load(Ordering::Relaxed));
        assert_eq!(lost.load(Ordering::Relaxed), 1);
        let config = config();
        let cancel = AtomicBool::new(false);
        let app = App::new(model("text"), None, None, &config, &cancel).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(40, 4)).unwrap();
        terminal
            .draw(|frame| {
                render(frame, &app, "fixture", &config, 257, &cancel);
            })
            .unwrap();
        assert!(contains(&terminal, "cola: 257 descartados"));
    }
}
