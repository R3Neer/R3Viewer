//! Text-only terminal session. The caller owns sanitizing document controls.
use std::fs::{File, OpenOptions};
use std::io::{self, IsTerminal, StdoutLock, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};

const HELP: &[&str] = &[
    "R3Viewer - texto",
    "",
    "Flechas / h j k l: desplazamiento",
    "PageUp / PageDown: una pagina",
    "Home: principio; End: final",
    "?: mostrar u ocultar esta ayuda",
    "q / Escape: salir",
    "Ctrl-C: cancelar (codigo 130)",
];

pub(crate) fn control_terminal() -> io::Result<File> {
    #[cfg(windows)]
    let path = "CONIN$";
    #[cfg(unix)]
    let path = "/dev/tty";
    #[cfg(not(any(windows, unix)))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "terminal de control no disponible",
    ));

    #[cfg(any(windows, unix))]
    {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        if !file.is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "entrada de control no es un terminal",
            ));
        }
        Ok(file)
    }
}

/// Detects the keyboard terminal only; never redirects output to CONOUT$/tty.
/// Crossterm 0.29 events/raw mode use CONIN$ on Windows and /dev/tty on Unix
/// when stdin is a pipe, so document bytes never become keyboard input.
pub fn available() -> bool {
    control_terminal().is_ok()
}

#[cfg(windows)]
mod console_mode {
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::io::AsRawHandle;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetConsoleMode(handle: *mut c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: *mut c_void, mode: u32) -> i32;
    }

    pub fn get(handle: &impl AsRawHandle) -> io::Result<u32> {
        let mut mode = 0;
        // SAFETY: the live owner lends its console handle; mode is writable.
        if unsafe { GetConsoleMode(handle.as_raw_handle(), &mut mode) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(mode)
        }
    }

    pub fn set(handle: &impl AsRawHandle, mode: u32) -> io::Result<()> {
        // SAFETY: the owner keeps the handle live throughout this call.
        if unsafe { SetConsoleMode(handle.as_raw_handle(), mode) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub(crate) struct TerminalGuard<'a> {
    pub(crate) output: StdoutLock<'a>,
    _control: File,
    raw: bool,
    alternate: bool,
    hidden: bool,
    #[cfg(windows)]
    input_mode: u32,
    #[cfg(windows)]
    output_mode: u32,
    #[cfg(windows)]
    output_mode_changed: bool,
}

impl<'a> TerminalGuard<'a> {
    pub(crate) fn enter(stdout: &'a io::Stdout, control: File) -> io::Result<Self> {
        #[cfg(windows)]
        let input_mode = console_mode::get(&control)?;
        #[cfg(windows)]
        let output_mode = console_mode::get(stdout)?;
        let mut guard = Self {
            output: stdout.lock(),
            _control: control,
            raw: false,
            alternate: false,
            hidden: false,
            #[cfg(windows)]
            input_mode,
            #[cfg(windows)]
            output_mode,
            #[cfg(windows)]
            output_mode_changed: false,
        };
        // A flag precedes each operation: even a partially failed write/init
        // must be rolled back by Drop. Unix preserves an already owned raw mode.
        if !terminal::is_raw_mode_enabled()? {
            guard.raw = true;
            terminal::enable_raw_mode()?;
        }
        #[cfg(windows)]
        {
            // Crossterm may enable VT processing as part of the next command.
            guard.output_mode_changed = true;
        }
        guard.alternate = true;
        execute!(guard.output, EnterAlternateScreen)?;
        guard.hidden = true;
        execute!(guard.output, Hide)?;
        Ok(guard)
    }

    pub(crate) fn restore(&mut self) -> io::Result<()> {
        let mut first_error = None;
        let mut record = |result: io::Result<()>| {
            if let Err(error) = result
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        };
        // Keep failed flags set so Drop gets one more chance to restore them.
        if self.hidden {
            let result = execute!(self.output, Show);
            self.hidden = result.is_err();
            record(result);
        }
        if self.alternate {
            let result = execute!(self.output, LeaveAlternateScreen);
            self.alternate = result.is_err();
            record(result);
        }
        if self.raw {
            #[cfg(windows)]
            let result = console_mode::set(&self._control, self.input_mode);
            #[cfg(not(windows))]
            let result = terminal::disable_raw_mode();
            self.raw = result.is_err();
            record(result);
        }
        #[cfg(windows)]
        if self.output_mode_changed && !self.alternate && !self.hidden {
            let result = console_mode::set(&io::stdout(), self.output_mode);
            self.output_mode_changed = result.is_err();
            record(result);
        }
        record(self.output.flush());
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for TerminalGuard<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Position {
    top: usize,
    left: usize,
}

impl Position {
    fn navigate(&mut self, key: KeyCode, lines: usize, width: usize, rows: usize, columns: usize) {
        // Resize keeps the exact document anchor. If a larger viewport now
        // extends beyond EOF, scroll back gradually rather than snapping it.
        let bottom = lines.saturating_sub(rows.max(1)).max(self.top);
        let right = width.saturating_sub(columns.max(1)).max(self.left);
        match key {
            KeyCode::Up | KeyCode::Char('k') => self.top = self.top.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.top = self.top.saturating_add(1).min(bottom),
            KeyCode::Left | KeyCode::Char('h') => self.left = self.left.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                self.left = self.left.saturating_add(1).min(right)
            }
            KeyCode::PageUp => self.top = self.top.saturating_sub(rows.max(1)),
            KeyCode::PageDown => self.top = self.top.saturating_add(rows.max(1)).min(bottom),
            KeyCode::Home => *self = Self::default(),
            KeyCode::End => self.top = lines.saturating_sub(rows.max(1)),
            _ => {}
        }
    }
}
#[cfg(test)]
fn slice_cells(line: &str, left: usize, columns: usize) -> String {
    crate::presentation::slice_cells(line, left, columns)
}
#[cfg(test)]
fn document<'a>(text: &'a str, cancelled: &AtomicBool) -> Option<(Vec<&'a str>, usize)> {
    let mut work = crate::text_engine::Work::preparation(cancelled);
    let index = crate::text_engine::Index::prepare(
        text,
        crate::text_engine::Identity::new(),
        &mut work,
        4096,
        usize::MAX,
    )
    .ok()?;
    Some((
        text.split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .collect(),
        index.max_width,
    ))
}
fn prepared(text: &str, cancel: &AtomicBool) -> io::Result<crate::presentation::SafeTextModel> {
    if text.len() > 128 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "fuente supera 128MiB",
        ));
    }
    let mut copy = String::new();
    copy.try_reserve_exact(text.len())
        .map_err(|_| io::Error::other("memoria de fuente"))?;
    if copy.capacity() > 128 * 1024 * 1024 {
        return Err(io::Error::other("capacidad de fuente"));
    }
    let mut at = 0;
    while at < text.len() {
        if cancel.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelado"));
        }
        let mut end = (at + 4096).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        copy.push_str(&text[at..end]);
        at = end;
    }
    crate::presentation::SafeTextModel::new(copy, cancel).map_err(|error| match error {
        crate::presentation::PrepareError::Cancelled => {
            io::Error::new(io::ErrorKind::Interrupted, "cancelado")
        }
        _ => io::Error::other(format!("{error:?}")),
    })
}
fn admitted_lines(
    model: &crate::presentation::SafeTextModel,
    domain: crate::viewport_admission::Domain,
    top: usize,
    rows: usize,
    left: usize,
    columns: usize,
    frame: &mut crate::viewport_admission::FrameBudget<'_>,
) -> io::Result<Vec<ratatui::text::Line<'static>>> {
    if let Some(reference) = &frame.failed {
        return Ok(vec![ratatui::text::Line::raw(
            crate::presentation::slice_cells(&reference.message(), 0, columns),
        )]);
    }
    let result = crate::viewport_admission::TextPlan::prepare(
        model,
        domain,
        top,
        rows,
        left..left.saturating_add(columns),
        frame,
    )
    .and_then(|plan| {
        plan.materialize(model, frame.work.cancel, |_, _| {
            ratatui::style::Style::default()
        })
    });
    match result {
        Ok(lines) => Ok(lines),
        Err(crate::viewport_admission::TextError::Reference(reference)) => {
            let line = ratatui::text::Line::raw(crate::presentation::slice_cells(
                &reference.message(),
                0,
                columns,
            ));
            frame.failed = Some(*reference);
            Ok(vec![line])
        }
        Err(crate::viewport_admission::TextError::Prepare(
            crate::presentation::PrepareError::Cancelled,
        )) => Err(io::Error::new(io::ErrorKind::Interrupted, "cancelado")),
        Err(error) => Err(io::Error::other(format!("{error:?}"))),
    }
}
struct LegacyDocuments<'a> {
    model: &'a crate::presentation::SafeTextModel,
    title: &'a str,
    help_model: &'a crate::presentation::SafeTextModel,
}
fn draw(
    output: &mut impl Write,
    documents: LegacyDocuments<'_>,
    position: &Position,
    size: (u16, u16),
    help: bool,
    cancel: &AtomicBool,
) -> io::Result<()> {
    let LegacyDocuments {
        model,
        title,
        help_model,
    } = documents;
    let columns = usize::from(size.0.saturating_sub(1));
    let rows = usize::from(size.1.saturating_sub(1));
    let mut frame = crate::viewport_admission::FrameBudget::new(cancel);
    // Prepare every body/status symbol before the first Print.
    let source = if help { help_model } else { model };
    let top = if help { 0 } else { position.top };
    let left = if help { 0 } else { position.left };
    let body = admitted_lines(
        source,
        crate::viewport_admission::Domain::Source,
        top,
        rows,
        left,
        columns,
        &mut frame,
    )?;
    // Descriptor preview uses the same frame reservation; control spelling
    // matches the legacy filename sanitizer, before copying or Print.
    let title_text = frame.label(title, columns);
    let temporary = title_text.len() * 6 + 8192;
    if frame.failed.is_none() && frame.reserve(temporary).is_err() {
        frame.preview_failure(
            0..title_text.len(),
            crate::viewport_admission::Reason::FrameSymbols,
        );
    }
    let status = prepared(
        &format!(
            "{title_text} | linea {}/{} col {} | ? ayuda | q salir",
            position.top + 1,
            model.line_count(),
            position.left + 1
        ),
        cancel,
    )?;
    let status = admitted_lines(
        &status,
        crate::viewport_admission::Domain::Preview,
        0,
        1,
        0,
        columns,
        &mut frame,
    )?;
    if cancel.load(Ordering::Relaxed) {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelado"));
    }
    queue!(output, MoveTo(0, 0), Clear(ClearType::All))?;
    let body = if let Some(reference) = &frame.failed {
        vec![ratatui::text::Line::raw(crate::presentation::slice_cells(
            &reference.message(),
            0,
            columns,
        ))]
    } else {
        body
    };
    for (row, line) in body.into_iter().enumerate() {
        queue!(output, MoveTo(0, row as u16))?;
        for span in line.spans {
            queue!(output, Print(span.content))?;
        }
    }
    if size.1 > 0 {
        queue!(output, MoveTo(0, size.1 - 1))?;
        for line in status {
            for span in line.spans {
                queue!(output, Print(span.content))?;
            }
        }
    }
    output.flush()
}
/// Runs on stdout only. `true` denotes cancellation; q/Escape return `false`.
/// Failures and unwinding restore terminal state through the owning guard.
pub fn run(text: &str, title: &str, cancelled: &AtomicBool) -> io::Result<bool> {
    if cancelled.load(Ordering::Relaxed) {
        return Ok(true);
    }
    let stdout = io::stdout();
    if !stdout.is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "la sesion requiere stdout terminal",
        ));
    }
    let control = control_terminal()?;
    let model = match prepared(text, cancelled) {
        Ok(model) => model,
        Err(error) if error.kind() == io::ErrorKind::Interrupted => return Ok(true),
        Err(error) => return Err(error),
    };
    let width = model.max_width();
    let help_model = prepared(&HELP.join("\n"), cancelled)?;
    let mut guard = TerminalGuard::enter(&stdout, control)?;
    let mut size = terminal::size()?;
    let mut position = Position::default();
    let mut help = false;
    let mut redraw = true;
    let was_cancelled = loop {
        if cancelled.load(Ordering::Relaxed) {
            break true;
        }
        if redraw {
            match draw(
                &mut guard.output,
                LegacyDocuments {
                    model: &model,
                    title,
                    help_model: &help_model,
                },
                &position,
                size,
                help,
                cancelled,
            ) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::Interrupted => break true,
                Err(error) => return Err(error),
            };
            redraw = false;
        }
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        match event::read()? {
            Event::Resize(columns, rows) => {
                size = (columns, rows);
                // Viewport dimensions change; document anchors remain valid.
                redraw = true;
            }
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char('c' | 'C'))
                {
                    cancelled.store(true, Ordering::Relaxed);
                    break true;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break false,
                    KeyCode::Char('?') => help = !help,
                    _ => position.navigate(
                        key.code,
                        model.line_count(),
                        width,
                        usize::from(size.1.saturating_sub(1)),
                        usize::from(size.0.saturating_sub(1)),
                    ),
                }
                redraw = true;
            }
            _ => {}
        }
    };
    guard.restore()?;
    Ok(was_cancelled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_viewports_keep_whole_graphemes_and_cells() {
        assert_eq!(slice_cells("A界B", 0, 3), "A界");
        assert_eq!(slice_cells("A界B", 0, 2), "A ");
        assert_eq!(slice_cells("A界B", 2, 2), " B");
        assert_eq!(slice_cells("e\u{301}xy", 0, 1), "e\u{301}");
        assert_eq!(slice_cells("e\u{301}xy", 1, 1), "x");
        assert_eq!(slice_cells("👩‍💻x", 0, 2), "👩‍💻");
        assert_eq!(slice_cells("👩‍💻x", 1, 2), " x");
        assert_eq!(slice_cells("🇪🇸x", 0, 2), "🇪🇸");
        assert_eq!(slice_cells("abc", 0, 0), "");
        assert_eq!(slice_cells("abc", 8, 4), "");
        assert_eq!(slice_cells("\u{301}x", 0, 1), "x");
    }

    #[test]
    fn tabs_preserve_original_column_alignment_when_scrolled() {
        assert_eq!(slice_cells("a\tb", 0, 9), "a       b");
        assert_eq!(slice_cells("a\tb", 6, 3), "  b");
        assert_eq!(slice_cells("界\tx", 1, 8), "       x");
    }

    #[test]
    fn navigation_preserves_resize_anchor_without_underflow() {
        let mut position = Position::default();
        position.navigate(KeyCode::PageUp, 30, 100, 10, 20);
        assert_eq!(position, Position::default());
        position.navigate(KeyCode::PageDown, 30, 100, 10, 20);
        assert_eq!(position.top, 10);
        position.navigate(KeyCode::End, 30, 100, 10, 20);
        assert_eq!(position.top, 20);
        position.left = 79;
        // Enlarging the viewport preserves the anchor, including when the
        // next key moves the other axis or moves back toward the beginning.
        position.navigate(KeyCode::Left, 30, 100, 25, 90);
        assert_eq!(position, Position { top: 20, left: 78 });
        position.navigate(KeyCode::Up, 30, 100, 25, 90);
        assert_eq!(position, Position { top: 19, left: 78 });
        position.navigate(KeyCode::PageDown, 30, 100, 25, 90);
        assert_eq!(position, Position { top: 19, left: 78 });
        position.navigate(KeyCode::Home, 30, 100, 0, 0);
        assert_eq!(position, Position::default());
        position.navigate(KeyCode::End, 0, 0, 0, 0);
        assert_eq!(position, Position::default());
    }

    #[test]
    fn crlf_empty_and_cancelled_documents() {
        let cancelled = AtomicBool::new(false);
        assert_eq!(
            document("a\r\nb\r\n", &cancelled),
            Some((vec!["a", "b", ""], 1))
        );
        assert_eq!(document("", &cancelled), Some((vec![""], 0)));
        cancelled.store(true, Ordering::Relaxed);
        assert_eq!(document("", &cancelled), None);
    }

    #[test]
    fn unicode_legacy_draw_preflights_complete_32mb_cluster_before_print() {
        let cancel = AtomicBool::new(false);
        let text = format!("ordinary\na{}\nafter", "\u{301}".repeat(16_000_000));
        let model = prepared(&text, &cancel).unwrap();
        let title = "fixture";
        let help_model = prepared("help", &cancel).unwrap();
        for (top, expected) in [(0, "ordinary"), (1, "referencia de viewport"), (2, "after")] {
            let mut output = Vec::new();
            draw(
                &mut output,
                LegacyDocuments {
                    model: &model,
                    title,
                    help_model: &help_model,
                },
                &Position { top, left: 0 },
                (120, 2),
                false,
                &cancel,
            )
            .unwrap();
            assert!(output.len() < 16384);
            assert!(String::from_utf8(output).unwrap().contains(expected));
        }
        assert_eq!(model.original(), text);
    }
}
