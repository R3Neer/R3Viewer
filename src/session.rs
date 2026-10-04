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
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const TAB_STOP: usize = 8;
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

fn control_terminal() -> io::Result<File> {
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

struct TerminalGuard<'a> {
    output: StdoutLock<'a>,
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
    fn enter(stdout: &'a io::Stdout, control: File) -> io::Result<Self> {
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

    fn restore(&mut self) -> io::Result<()> {
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
fn grapheme_width(grapheme: &str, column: usize) -> usize {
    if grapheme == "\t" {
        TAB_STOP - column % TAB_STOP
    } else {
        UnicodeWidthStr::width(grapheme)
    }
}

/// Produces a viewport in display cells, never slicing UTF-8 or a grapheme.
/// Partial wide glyphs become spaces so subsequent glyphs stay in their cells.
fn slice_cells(line: &str, left: usize, columns: usize) -> String {
    let right = left.saturating_add(columns);
    let mut column = 0usize;
    let mut output = String::new();
    for grapheme in line.graphemes(true) {
        if column >= right {
            break;
        }
        let width = grapheme_width(grapheme, column);
        let end = column.saturating_add(width);
        if width > 0 && end > left {
            let visible = end.min(right).saturating_sub(column.max(left));
            if grapheme == "\t" || column < left || end > right {
                output.extend(std::iter::repeat_n(' ', visible));
            } else {
                output.push_str(grapheme);
            }
        }
        // Standalone zero-width clusters are omitted rather than allowing a
        // combining mark to alter a cell preceding this viewport.
        column = end;
    }
    output
}

fn document<'a>(text: &'a str, cancelled: &AtomicBool) -> Option<(Vec<&'a str>, usize)> {
    let mut lines = Vec::new();
    let mut max_width = 0;
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let mut width = 0;
        for (index, grapheme) in line.graphemes(true).enumerate() {
            if index % 4096 == 0 && cancelled.load(Ordering::Relaxed) {
                return None;
            }
            width += grapheme_width(grapheme, width);
        }
        if cancelled.load(Ordering::Relaxed) {
            return None;
        }
        max_width = max_width.max(width);
        lines.push(line);
    }
    Some((lines, max_width))
}

fn draw(
    output: &mut impl Write,
    lines: &[&str],
    title: &str,
    position: &Position,
    size: (u16, u16),
    help: bool,
) -> io::Result<()> {
    // Reserve the last column: printing in the bottom-right cell can scroll
    // terminals with autowrap. Do not change the user's autowrap setting.
    let columns = usize::from(size.0.saturating_sub(1));
    let rows = usize::from(size.1.saturating_sub(1));
    queue!(output, MoveTo(0, 0), Clear(ClearType::All))?;
    let source = if help { HELP } else { lines };
    let top = if help { 0 } else { position.top };
    let left = if help { 0 } else { position.left };
    for (row, line) in source.iter().skip(top).take(rows).enumerate() {
        queue!(
            output,
            MoveTo(0, row as u16),
            Print(slice_cells(line, left, columns))
        )?;
    }
    if size.1 > 0 {
        let status = format!(
            "{title} | linea {}/{} col {} | ? ayuda | q salir",
            position.top + 1,
            lines.len(),
            position.left + 1
        );
        queue!(
            output,
            MoveTo(0, size.1 - 1),
            Print(slice_cells(&status, 0, columns))
        )?;
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
    let Some((lines, width)) = document(text, cancelled) else {
        return Ok(true);
    };
    // Titles may originate in filenames; the document sanitizer does not cover
    // them. Prevent even filenames containing terminal controls from executing.
    let title: String = title
        .chars()
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect();
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
            draw(&mut guard.output, &lines, &title, &position, size, help)?;
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
                        lines.len(),
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
}
