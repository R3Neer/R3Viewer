//! Borrowed inspection of typed input. Views prepare column metadata once;
//! rows and cells are read lazily from the retained source model.
use crate::text::safe_text;
use r3viewer_wire::{ByteStreamKind, Payload, TypedInput, Value};
use std::collections::HashSet;
use std::fmt;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PathMember {
    Key(String),
    Index(usize),
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct LogicalPath(pub Vec<PathMember>);
impl LogicalPath {
    /// Bounded, safe breadcrumb label. Indices are always zero based.
    pub fn label(&self, max_chars: usize) -> String {
        let mut label = String::from("$");
        for member in &self.0 {
            if label.chars().count() >= max_chars {
                label.push('…');
                break;
            }
            match member {
                PathMember::Key(key) => {
                    label.push('[');
                    label.push_str(&format!("{:?}", prefix(key, max_chars)));
                    label.push(']');
                }
                PathMember::Index(index) => label.push_str(&format!("[{index}]")),
            }
        }
        cap_safe(&label, max_chars)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum DataRef<'a> {
    Absent,
    Value(&'a Value),
    ListStream(&'a [Value]),
    ByteStream {
        kind: ByteStreamKind,
        bytes: &'a [u8],
    },
}
impl<'a> DataRef<'a> {
    pub fn root(input: &'a TypedInput) -> Self {
        Self::from_payload(&input.payload)
    }
    pub fn from_payload(payload: &'a Payload) -> Self {
        match payload {
            Payload::Absent => Self::Absent,
            Payload::SingleValue(value) => Self::Value(value),
            Payload::ListStream(values) => Self::ListStream(values),
            Payload::ByteStream { kind, bytes } => Self::ByteStream { kind: *kind, bytes },
        }
    }
    pub fn sequence(self) -> Option<&'a [Value]> {
        match self {
            Self::Value(Value::List(values)) => Some(values.as_slice()),
            Self::ListStream(values) => Some(values),
            _ => None,
        }
    }
    pub fn record(self) -> Option<&'a [(String, Value)]> {
        match self {
            Self::Value(Value::Record(entries)) => Some(entries),
            _ => None,
        }
    }
    pub fn child(self, member: &PathMember) -> Option<Self> {
        match member {
            PathMember::Index(index) => self.sequence()?.get(*index).map(Self::Value),
            PathMember::Key(key) => self
                .record()?
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| Self::Value(value)),
        }
    }
    pub fn resolve(self, path: &LogicalPath) -> Option<Self> {
        path.0
            .iter()
            .try_fold(self, |node, member| node.child(member))
    }
    fn path_limit(self) -> usize {
        r3viewer_wire::MAX_VALUE_DEPTH - usize::from(!matches!(self, Self::ListStream(_)))
    }
    pub fn is_container(self) -> bool {
        self.sequence().is_some() || self.record().is_some()
    }
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::ListStream(_) => "list_stream",
            Self::ByteStream {
                kind: ByteStreamKind::String,
                ..
            } => "string_stream",
            Self::ByteStream {
                kind: ByteStreamKind::Binary,
                ..
            } => "binary_stream",
            Self::ByteStream {
                kind: ByteStreamKind::Unknown,
                ..
            } => "unknown_stream",
            Self::Value(value) => match value {
                Value::Nothing => "nothing",
                Value::Bool(_) => "bool",
                Value::Int(_) => "int",
                Value::Float(_) => "float",
                Value::String(_) => "string",
                Value::Binary(_) => "binary",
                Value::Date(_) => "date",
                Value::Filesize(_) => "filesize",
                Value::Duration(_) => "duration",
                Value::Record(_) => "record",
                Value::List(_) => "list",
                Value::IntRange { .. } => "int_range",
                Value::FloatRange { .. } => "float_range",
                Value::CellPath(_) => "cell_path",
                Value::Glob { .. } => "glob",
                Value::Error { .. } => "error",
                Value::Closure => "closure",
            },
        }
    }
    /// Preview only: never used as the search domain. Work and allocation are
    /// bounded by max_chars, apart from constant-size primitive descriptors.
    pub fn summary(self, max_chars: usize) -> String {
        if max_chars == 0 {
            return String::new();
        }
        let name = self.type_name();
        let raw = match self {
            Self::Absent => "absent".into(),
            Self::ListStream(values) => format!("list_stream[{}]", values.len()),
            Self::ByteStream { kind, bytes } => {
                if kind == ByteStreamKind::String {
                    let bytes = &bytes[..bytes
                        .len()
                        .min(max_chars.saturating_mul(4).saturating_add(3))];
                    let valid = match std::str::from_utf8(bytes) {
                        Ok(s) => s,
                        Err(e) => std::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap_or(""),
                    };
                    format!("{name}({:?})", prefix(valid, max_chars))
                } else {
                    format!("{name}({} bytes; {})", bytes.len(), hex_preview(bytes, 8))
                }
            }
            Self::Value(value) => match value {
                Value::Nothing => "nothing".into(),
                Value::Bool(v) => format!("bool({v})"),
                Value::Int(v) => format!("int({v})"),
                Value::Float(v) => format!("float({v:?}; bits=0x{:016x})", v.to_bits()),
                Value::String(v) => format!("string({:?})", prefix(v, max_chars)),
                Value::Binary(bytes) => {
                    format!("binary({} bytes; {})", bytes.len(), hex_preview(bytes, 8))
                }
                Value::Date(v) => format!("date({})", prefix(v, max_chars)),
                Value::Filesize(v) => format!("filesize({v} bytes)"),
                Value::Duration(v) => format!("duration({v} ns)"),
                Value::Record(entries) => format!("record{{{} fields}}", entries.len()),
                Value::List(values) => format!("list[{}]", values.len()),
                Value::IntRange { start, step, end } => {
                    format!("int_range(start={start}, step={step}, end={end:?})")
                }
                Value::FloatRange { start, step, end } => {
                    format!("float_range(start={start:?}, step={step:?}, end={end:?})")
                }
                Value::CellPath(members) => format!("cell_path({} members)", members.len()),
                Value::Glob { pattern, no_expand } => format!(
                    "glob({:?}; no_expand={no_expand})",
                    prefix(pattern, max_chars)
                ),
                Value::Error { message } => format!("error({:?})", prefix(message, max_chars)),
                Value::Closure => "closure(opaque)".into(),
            },
        };
        cap_safe(&raw, max_chars)
    }
    pub fn visits(self, cancel: &'a AtomicBool) -> Visits<'a> {
        Visits::new(self, cancel)
    }
}
fn prefix(text: &str, max_chars: usize) -> &str {
    match text.char_indices().nth(max_chars) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}
fn cap_safe(text: &str, max_chars: usize) -> String {
    // All callers supply an already bounded preview, never a whole large leaf.
    let safe = safe_text(text);
    let mut chars = safe.chars();
    let head: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_none() {
        head
    } else if max_chars == 0 {
        String::new()
    } else {
        let mut head: String = head.chars().take(max_chars - 1).collect();
        head.push('…');
        head
    }
}
fn hex_preview(bytes: &[u8], count: usize) -> String {
    let mut result = String::from("0x");
    for byte in bytes.iter().take(count) {
        result.push_str(&format!("{byte:02x}"));
    }
    if bytes.len() > count {
        result.push('…');
    }
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataError {
    Cancelled,
    InvalidPath,
    TooDeep,
}
impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "data preparation cancelled",
            Self::InvalidPath => "invalid data path",
            Self::TooDeep => "data path exceeds depth limit",
        })
    }
}
impl std::error::Error for DataError {}
pub type Result<T> = std::result::Result<T, DataError>;
fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(DataError::Cancelled)
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
    pub top: usize,
    pub left: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreparationStats {
    pub rows_scanned: usize,
    pub fields_scanned: usize,
}
#[derive(Debug)]
enum Layout<'a> {
    Record(&'a [(String, Value)]),
    List(&'a [Value]),
    Table {
        rows: &'a [Value],
        columns: Vec<&'a str>,
    },
    Leaf(DataRef<'a>),
}
#[derive(Debug)]
pub struct ViewState<'a> {
    pub path: LogicalPath,
    pub cursor: Cursor,
    pub preparation: PreparationStats,
    node: DataRef<'a>,
    layout: Layout<'a>,
}
#[derive(Clone, Copy, Debug)]
pub enum Cell<'a> {
    Missing,
    Key(&'a str),
    Index(usize),
    Data(DataRef<'a>),
}
impl Cell<'_> {
    pub fn summary(self, max_chars: usize) -> String {
        match self {
            Self::Missing => cap_safe("missing", max_chars),
            Self::Key(key) => {
                let head = prefix(key, max_chars);
                if head.len() == key.len() {
                    cap_safe(head, max_chars)
                } else {
                    cap_safe(&format!("{head}…"), max_chars)
                }
            }
            Self::Index(index) => cap_safe(&index.to_string(), max_chars),
            Self::Data(value) => value.summary(max_chars),
        }
    }
}
impl<'a> ViewState<'a> {
    fn prepare(node: DataRef<'a>, path: LogicalPath, cancel: &AtomicBool) -> Result<Self> {
        check(cancel)?;
        let mut preparation = PreparationStats::default();
        let layout = if let Some(record) = node.record() {
            Layout::Record(record)
        } else if let Some(rows) = node.sequence() {
            let mut records = !rows.is_empty();
            let mut columns = Vec::new();
            let mut seen = HashSet::new();
            for row in rows {
                check(cancel)?;
                preparation.rows_scanned += 1;
                let Value::Record(entries) = row else {
                    records = false;
                    break;
                };
                for (key, _) in entries {
                    check(cancel)?;
                    preparation.fields_scanned += 1;
                    if seen.insert(key.as_str()) {
                        columns.push(key.as_str());
                    }
                }
            }
            if records {
                Layout::Table { rows, columns }
            } else {
                Layout::List(rows)
            }
        } else {
            Layout::Leaf(node)
        };
        check(cancel)?;
        Ok(Self {
            path,
            cursor: Cursor::default(),
            preparation,
            node,
            layout,
        })
    }
    pub fn node(&self) -> DataRef<'a> {
        self.node
    }
    pub fn is_table(&self) -> bool {
        matches!(self.layout, Layout::Table { .. })
    }
    pub fn row_count(&self) -> usize {
        match &self.layout {
            Layout::Record(entries) => entries.len(),
            Layout::List(rows) | Layout::Table { rows, .. } => rows.len(),
            Layout::Leaf(_) => 1,
        }
    }
    pub fn column_count(&self) -> usize {
        match &self.layout {
            Layout::Record(_) | Layout::List(_) => 2,
            Layout::Table { columns, .. } => columns.len().max(1),
            Layout::Leaf(_) => 1,
        }
    }
    /// Table labels retain the exact key. Sanitize only at presentation time.
    pub fn column_name(&self, column: usize) -> Option<&str> {
        match &self.layout {
            Layout::Record(_) => ["key", "value"].get(column).copied(),
            Layout::List(_) => ["index", "value"].get(column).copied(),
            Layout::Table { columns, .. } if columns.is_empty() && column == 0 => Some("record"),
            Layout::Table { columns, .. } => columns.get(column).copied(),
            Layout::Leaf(_) if column == 0 => Some("value"),
            _ => None,
        }
    }
    /// Exact table key, excluding the synthetic empty-record fallback column.
    pub fn column_key(&self, column: usize) -> Option<&'a str> {
        match &self.layout {
            Layout::Table { columns, .. } => columns.get(column).copied(),
            _ => None,
        }
    }
    pub fn cell(&self, row: usize, column: usize) -> Option<Cell<'a>> {
        if row >= self.row_count() || column >= self.column_count() {
            return None;
        }
        Some(match &self.layout {
            Layout::Record(entries) => {
                if column == 0 {
                    Cell::Key(entries[row].0.as_str())
                } else {
                    Cell::Data(DataRef::Value(&entries[row].1))
                }
            }
            Layout::List(rows) => {
                if column == 0 {
                    Cell::Index(row)
                } else {
                    Cell::Data(DataRef::Value(&rows[row]))
                }
            }
            Layout::Table { rows, columns } if columns.is_empty() => {
                Cell::Data(DataRef::Value(&rows[row]))
            }
            Layout::Table { rows, columns } => {
                let Value::Record(entries) = &rows[row] else {
                    unreachable!()
                };
                match entries.iter().find(|(key, _)| key == columns[column]) {
                    Some((_, value)) => Cell::Data(DataRef::Value(value)),
                    None => Cell::Missing,
                }
            }
            Layout::Leaf(value) => Cell::Data(*value),
        })
    }
    pub fn cell_path(&self, row: usize, column: usize) -> Option<LogicalPath> {
        self.cell(row, column)?;
        let mut path = self.path.clone();
        match &self.layout {
            Layout::Record(entries) => path.0.push(PathMember::Key(entries[row].0.clone())),
            Layout::List(_) => path.0.push(PathMember::Index(row)),
            Layout::Table { columns, .. } => {
                if matches!(self.cell(row, column), Some(Cell::Missing)) {
                    return None;
                }
                path.0.push(PathMember::Index(row));
                if !columns.is_empty() {
                    path.0.push(PathMember::Key(columns[column].into()));
                }
            }
            Layout::Leaf(_) => return None,
        }
        Some(path)
    }
    /// Only the requested row range is iterated. No rows or cells are collected.
    pub fn rows(&self, range: Range<usize>) -> impl Iterator<Item = RowRef<'_, 'a>> {
        let end = range.end.min(self.row_count());
        (range.start.min(end)..end).map(|index| RowRef { view: self, index })
    }
    pub fn selected_path(&self) -> Option<LogicalPath> {
        self.cell_path(self.cursor.row, self.cursor.column)
    }
    pub fn clamp_cursor(&mut self) {
        self.cursor.row = self.cursor.row.min(self.row_count().saturating_sub(1));
        self.cursor.column = self
            .cursor
            .column
            .min(self.column_count().saturating_sub(1));
        self.cursor.top = self.cursor.top.min(self.cursor.row);
        self.cursor.left = self.cursor.left.min(self.cursor.column);
    }
}
#[derive(Clone, Copy, Debug)]
pub struct RowRef<'v, 'a> {
    view: &'v ViewState<'a>,
    pub index: usize,
}
impl<'a> RowRef<'_, 'a> {
    pub fn cell(&self, column: usize) -> Option<Cell<'a>> {
        self.view.cell(self.index, column)
    }
}

#[derive(Debug)]
pub struct Navigator<'a> {
    root: DataRef<'a>,
    stack: Vec<ViewState<'a>>,
}
impl<'a> Navigator<'a> {
    pub fn new(input: &'a TypedInput, cancel: &AtomicBool) -> Result<Self> {
        Self::from_root(DataRef::root(input), cancel)
    }
    pub fn from_root(root: DataRef<'a>, cancel: &AtomicBool) -> Result<Self> {
        Ok(Self {
            root,
            stack: vec![ViewState::prepare(root, LogicalPath::default(), cancel)?],
        })
    }
    pub fn root(&self) -> DataRef<'a> {
        self.root
    }
    pub fn current(&self) -> &ViewState<'a> {
        self.stack.last().expect("root view retained")
    }
    pub fn current_mut(&mut self) -> &mut ViewState<'a> {
        self.stack.last_mut().expect("root view retained")
    }
    pub fn breadcrumbs(&self) -> impl Iterator<Item = &LogicalPath> {
        self.stack.iter().map(|view| &view.path)
    }
    pub fn enter(&mut self, row: usize, column: usize, cancel: &AtomicBool) -> Result<bool> {
        check(cancel)?;
        let Some(path) = self.current().cell_path(row, column) else {
            return Ok(false);
        };
        if path.0.len() > self.root.path_limit() {
            return Err(DataError::TooDeep);
        }
        let node = self.root.resolve(&path).ok_or(DataError::InvalidPath)?;
        let view = ViewState::prepare(node, path, cancel)?;
        self.current_mut().cursor.row = row;
        self.current_mut().cursor.column = column;
        self.stack.push(view);
        Ok(true)
    }
    /// Open a complete sequence row, including a record whose table cells are
    /// all missing. The current cell selection and scroll survive Back.
    pub fn enter_row(&mut self, row: usize, cancel: &AtomicBool) -> Result<bool> {
        check(cancel)?;
        let Some(rows) = self.current().node().sequence() else {
            return Ok(false);
        };
        let Some(value) = rows.get(row) else {
            return Ok(false);
        };
        let mut path = self.current().path.clone();
        path.0.push(PathMember::Index(row));
        if path.0.len() > self.root.path_limit() {
            return Err(DataError::TooDeep);
        }
        let view = ViewState::prepare(DataRef::Value(value), path, cancel)?;
        self.current_mut().cursor.row = row;
        self.stack.push(view);
        Ok(true)
    }
    pub fn back(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }
    /// Open all ancestors of a search result. Preparation is transactional: a
    /// cancelled or invalid jump leaves the previous view and cursor intact.
    pub fn jump(&mut self, path: &LogicalPath, cancel: &AtomicBool) -> Result<()> {
        if path.0.len() > self.root.path_limit() {
            return Err(DataError::TooDeep);
        }
        check(cancel)?;
        let mut next = vec![ViewState::prepare(
            self.root,
            LogicalPath::default(),
            cancel,
        )?];
        let mut node = self.root;
        let mut prefix = LogicalPath::default();
        for member in &path.0 {
            check(cancel)?;
            let parent = next.last_mut().expect("root retained");
            match member {
                PathMember::Index(index) => {
                    parent.cursor.row = *index;
                    parent.cursor.column = usize::from(!parent.is_table());
                }
                PathMember::Key(key) => {
                    let record = node.record().ok_or(DataError::InvalidPath)?;
                    parent.cursor.row = record
                        .iter()
                        .position(|(name, _)| name == key)
                        .ok_or(DataError::InvalidPath)?;
                    parent.cursor.column = 1;
                }
            }
            node = node.child(member).ok_or(DataError::InvalidPath)?;
            prefix.0.push(member.clone());
            next.push(ViewState::prepare(node, prefix.clone(), cancel)?);
        }
        check(cancel)?;
        self.stack = next;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowedMember<'a> {
    Key(&'a str),
    Index(usize),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BorrowedPath<'a>(pub Vec<BorrowedMember<'a>>);
impl BorrowedPath<'_> {
    pub fn to_logical_path(&self) -> LogicalPath {
        LogicalPath(
            self.0
                .iter()
                .map(|member| match member {
                    BorrowedMember::Key(key) => PathMember::Key((*key).into()),
                    BorrowedMember::Index(index) => PathMember::Index(*index),
                })
                .collect(),
        )
    }
}
#[derive(Clone, Copy, Debug)]
pub enum VisitItem<'a> {
    Key(&'a str),
    Primitive(DataRef<'a>),
}
#[derive(Clone, Debug)]
pub struct SearchVisit<'a> {
    pub path: BorrowedPath<'a>,
    pub item: VisitItem<'a>,
}
#[derive(Debug)]
struct WalkFrame<'a> {
    node: DataRef<'a>,
    path: BorrowedPath<'a>,
    next_child: usize,
}
/// Depth-first traversal, one key or complete primitive per call. Containers
/// hold only an index into the original slice, never a queue of all children.
pub struct Visits<'a> {
    stack: Vec<WalkFrame<'a>>,
    pending: Option<WalkFrame<'a>>,
    cancel: &'a AtomicBool,
    done: bool,
    path_limit: usize,
}
impl<'a> Visits<'a> {
    fn new(root: DataRef<'a>, cancel: &'a AtomicBool) -> Self {
        Self {
            stack: vec![WalkFrame {
                node: root,
                path: BorrowedPath::default(),
                next_child: 0,
            }],
            pending: None,
            cancel,
            done: false,
            path_limit: root.path_limit(),
        }
    }
}
impl<'a> Iterator for Visits<'a> {
    type Item = Result<SearchVisit<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            if let Err(error) = check(self.cancel) {
                self.done = true;
                return Some(Err(error));
            }
            if let Some(child) = self.pending.take() {
                if child.path.0.len() > self.path_limit {
                    self.done = true;
                    return Some(Err(DataError::TooDeep));
                }
                self.stack.push(child);
            }
            let frame = self.stack.last_mut()?;
            if let Some(entries) = frame.node.record() {
                let Some((key, value)) = entries.get(frame.next_child) else {
                    self.stack.pop();
                    continue;
                };
                frame.next_child += 1;
                let mut path = frame.path.clone();
                path.0.push(BorrowedMember::Key(key));
                self.pending = Some(WalkFrame {
                    node: DataRef::Value(value),
                    path: path.clone(),
                    next_child: 0,
                });
                return Some(Ok(SearchVisit {
                    path,
                    item: VisitItem::Key(key),
                }));
            } else if let Some(values) = frame.node.sequence() {
                let Some(value) = values.get(frame.next_child) else {
                    self.stack.pop();
                    continue;
                };
                let index = frame.next_child;
                frame.next_child += 1;
                let mut path = frame.path.clone();
                path.0.push(BorrowedMember::Index(index));
                self.pending = Some(WalkFrame {
                    node: DataRef::Value(value),
                    path,
                    next_child: 0,
                });
            } else {
                let frame = self.stack.pop().expect("current frame");
                return Some(Ok(SearchVisit {
                    path: frame.path,
                    item: VisitItem::Primitive(frame.node),
                }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r3viewer_wire::{Bound, Origin, Start, VERSION};
    fn input(payload: Payload) -> TypedInput {
        let origin = match &payload {
            Payload::Absent => Origin::Absent,
            Payload::SingleValue(_) => Origin::SingleValue,
            Payload::ListStream(_) => Origin::ListStream,
            Payload::ByteStream { kind, .. } => Origin::ByteStream(*kind),
        };
        TypedInput {
            start: Start {
                version: VERSION,
                cwd: "unopened-fixture".into(),
                origin,
            },
            payload,
        }
    }
    fn no_cancel() -> AtomicBool {
        AtomicBool::new(false)
    }
    #[test]
    fn ordered_records_nested_navigation_and_back_preserve_cursor() {
        let input = input(Payload::SingleValue(Value::Record(vec![
            ("z".into(), Value::Int(7)),
            (
                "a".into(),
                Value::List(vec![
                    Value::Nothing,
                    Value::Record(vec![("m".into(), Value::String("text".into()))]),
                ]),
            ),
            ("m".into(), Value::Bool(false)),
        ])));
        let mut nav = Navigator::new(&input, &no_cancel()).unwrap();
        assert_eq!(nav.current().row_count(), 3);
        assert!(matches!(nav.current().cell(0, 0), Some(Cell::Key("z"))));
        assert!(matches!(nav.current().cell(1, 0), Some(Cell::Key("a"))));
        nav.current_mut().cursor = Cursor {
            row: 1,
            column: 1,
            top: 1,
            left: 1,
        };
        assert!(nav.enter(1, 1, &no_cancel()).unwrap());
        assert_eq!(
            nav.current().path,
            LogicalPath(vec![PathMember::Key("a".into())])
        );
        nav.current_mut().cursor = Cursor {
            row: 1,
            column: 1,
            top: 1,
            left: 0,
        };
        assert!(nav.enter(1, 1, &no_cancel()).unwrap());
        assert_eq!(nav.breadcrumbs().count(), 3);
        assert!(nav.back());
        assert_eq!(
            nav.current().cursor,
            Cursor {
                row: 1,
                column: 1,
                top: 1,
                left: 0
            }
        );
        assert!(nav.back());
        assert_eq!(
            nav.current().cursor,
            Cursor {
                row: 1,
                column: 1,
                top: 1,
                left: 1
            }
        );
        assert!(!nav.back());
    }
    #[test]
    fn table_columns_first_appearance_missing_nothing_and_empty_are_distinct() {
        let input = input(Payload::ListStream(vec![
            Value::Record(vec![
                ("z".into(), Value::Nothing),
                ("a".into(), Value::String(String::new())),
            ]),
            Value::Record(vec![
                ("m".into(), Value::Int(9)),
                ("a".into(), Value::Int(2)),
            ]),
        ]));
        let mut nav = Navigator::new(&input, &no_cancel()).unwrap();
        assert!(nav.current().is_table());
        assert_eq!(
            (0..3)
                .map(|col| nav.current().column_name(col).unwrap())
                .collect::<Vec<_>>(),
            ["z", "a", "m"]
        );
        assert!(matches!(
            nav.current().cell(0, 0),
            Some(Cell::Data(DataRef::Value(Value::Nothing)))
        ));
        assert!(matches!(nav.current().cell(1, 0), Some(Cell::Missing)));
        assert!(
            matches!(nav.current().cell(0,1),Some(Cell::Data(DataRef::Value(Value::String(v)))) if v.is_empty())
        );
        assert!(!nav.enter(1, 0, &no_cancel()).unwrap());
        assert!(nav.enter(1, 1, &no_cancel()).unwrap());
        assert_eq!(
            nav.current().path,
            LogicalPath(vec![PathMember::Index(1), PathMember::Key("a".into())])
        );
        assert!(matches!(
            nav.current().node(),
            DataRef::Value(Value::Int(2))
        ));
        assert!(nav.back());
        assert_eq!(nav.current().cursor.row, 1);
        assert_eq!(nav.current().cursor.column, 1);
    }
    #[test]
    fn heterogeneous_list_and_empty_origins_keep_their_identities() {
        let input = input(Payload::SingleValue(Value::List(vec![
            Value::Int(1),
            Value::Record(vec![]),
            Value::String("x".into()),
        ])));
        let nav = Navigator::new(&input, &no_cancel()).unwrap();
        assert!(!nav.current().is_table());
        assert_eq!(nav.current().row_count(), 3);
        assert!(matches!(nav.current().cell(2, 0), Some(Cell::Index(2))));
        let absent = input_payload_ref(Payload::Absent);
        let nothing = input_payload_ref(Payload::SingleValue(Value::Nothing));
        let empty = input_payload_ref(Payload::SingleValue(Value::List(vec![])));
        assert_eq!(absent, "absent");
        assert_eq!(nothing, "nothing");
        assert_eq!(empty, "list[0]");
    }
    fn input_payload_ref(payload: Payload) -> String {
        DataRef::from_payload(&payload).summary(100)
    }
    #[test]
    fn hundred_thousand_rows_are_prepared_once_and_viewport_borrows_only_requested_cells() {
        let rows: Vec<_> = (0..100_000)
            .map(|n| Value::Record(vec![("n".into(), Value::Int(n))]))
            .collect();
        let input = input(Payload::SingleValue(Value::List(rows)));
        let nav = Navigator::new(&input, &no_cancel()).unwrap();
        assert_eq!(nav.current().preparation.rows_scanned, 100_000);
        assert_eq!(nav.current().preparation.fields_scanned, 100_000);
        let DataRef::Value(Value::List(source)) = nav.root() else {
            panic!()
        };
        let Value::Record(expected) = &source[99_985] else {
            panic!()
        };
        let mut accesses = 0;
        for row in nav.current().rows(99_980..99_990) {
            accesses += 1;
            let Cell::Data(DataRef::Value(value)) = row.cell(0).unwrap() else {
                panic!()
            };
            if row.index == 99_985 {
                assert!(std::ptr::eq(value, &expected[0].1));
            }
        }
        assert_eq!(accesses, 10);
        assert_eq!(nav.current().rows(99_998..usize::MAX).count(), 2);
        assert_eq!(nav.current().preparation.rows_scanned, 100_000);
        assert_eq!(nav.current().rows(usize::MAX..usize::MAX).count(), 0);
    }
    #[test]
    fn collapsed_search_visits_keys_and_full_primitives_without_copying_subtrees() {
        let input = input(Payload::SingleValue(Value::Record(vec![(
            "outer".into(),
            Value::List(vec![Value::Record(vec![(
                "needle-key".into(),
                Value::String(format!("{}needle-end", "x".repeat(200_000))),
            )])]),
        )])));
        let cancel = no_cancel();
        let mut visits = DataRef::root(&input).visits(&cancel);
        let mut keys = Vec::new();
        let mut primitive_path = None;
        for visit in visits.by_ref() {
            let visit = visit.unwrap();
            match visit.item {
                VisitItem::Key(key) => keys.push(key),
                VisitItem::Primitive(DataRef::Value(Value::String(value))) => {
                    assert!(value.ends_with("needle-end"));
                    let expected_path = LogicalPath(vec![
                        PathMember::Key("outer".into()),
                        PathMember::Index(0),
                        PathMember::Key("needle-key".into()),
                    ]);
                    assert_eq!(visit.path.to_logical_path(), expected_path);
                    let DataRef::Value(Value::String(source)) =
                        DataRef::root(&input).resolve(&expected_path).unwrap()
                    else {
                        panic!()
                    };
                    assert!(std::ptr::eq(source, value));
                    primitive_path = Some(expected_path);
                }
                _ => panic!(),
            }
        }
        assert_eq!(keys, ["outer", "needle-key"]);
        let mut nav = Navigator::new(&input, &cancel).unwrap();
        nav.jump(&primitive_path.unwrap(), &cancel).unwrap();
        assert_eq!(nav.breadcrumbs().count(), 4);
    }
    #[test]
    fn cancellation_invalid_jumps_and_descriptors_never_execute_or_mutate_history() {
        let input = input(Payload::SingleValue(Value::List(vec![
            Value::Closure,
            Value::IntRange {
                start: 0,
                step: 1,
                end: Bound::Unbounded,
            },
            Value::Glob {
                pattern: "*".into(),
                no_expand: false,
            },
        ])));
        let cancel = no_cancel();
        let mut nav = Navigator::new(&input, &cancel).unwrap();
        nav.current_mut().cursor = Cursor {
            row: 2,
            column: 1,
            top: 2,
            left: 0,
        };
        let before = nav.current().cursor;
        assert!(matches!(
            nav.jump(&LogicalPath(vec![PathMember::Index(99)]), &cancel),
            Err(DataError::InvalidPath)
        ));
        assert_eq!(nav.current().cursor, before);
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            nav.enter(0, 1, &cancel),
            Err(DataError::Cancelled)
        ));
        assert_eq!(nav.current().cursor, before);
        assert!(matches!(
            DataRef::root(&input).visits(&cancel).next(),
            Some(Err(DataError::Cancelled))
        ));
        cancel.store(false, Ordering::Relaxed);
        let descriptors: Vec<_> = DataRef::root(&input)
            .visits(&cancel)
            .map(|visit| match visit.unwrap().item {
                VisitItem::Primitive(node) => node.summary(200),
                VisitItem::Key(_) => unreachable!(),
            })
            .collect();
        assert!(descriptors[0].contains("opaque"));
        assert!(descriptors[1].contains("Unbounded"));
        assert!(descriptors[2].contains("no_expand=false"));
    }
    #[test]
    fn summaries_are_bounded_and_cannot_contain_executable_controls() {
        let value = Value::String(format!("\x1b]0;attack\x07{}", "界".repeat(1_000_000)));
        let preview = DataRef::Value(&value).summary(40);
        assert!(preview.chars().count() <= 40);
        assert!(!preview.chars().any(|c| c.is_control()));
        assert!(preview.ends_with('…'));
        let key = Cell::Key("\x1b[31mkey\u{009b}31m").summary(24);
        assert!(!key.contains('\x1b'));
        assert!(!key.contains('\u{009b}'));
        assert!(DataRef::Value(&value).summary(0).is_empty());
        let binary = Value::Binary(vec![255; 1_000_000]);
        assert!(
            DataRef::Value(&binary)
                .summary(80)
                .contains("1000000 bytes")
        );
    }
    #[test]
    fn valid_wire_depth_is_accessible_for_single_values_and_stream_elements() {
        fn nested(depth: usize) -> Value {
            let mut value = Value::Int(9);
            for _ in 1..depth {
                value = Value::List(vec![value]);
            }
            value
        }
        let single = input(Payload::SingleValue(nested(r3viewer_wire::MAX_VALUE_DEPTH)));
        let stream = input(Payload::ListStream(vec![nested(
            r3viewer_wire::MAX_VALUE_DEPTH,
        )]));
        let cancel = no_cancel();
        for (source, path_len) in [(&single, 63), (&stream, 64)] {
            let mut nav = Navigator::new(source, &cancel).unwrap();
            let path = LogicalPath(vec![PathMember::Index(0); path_len]);
            nav.jump(&path, &cancel).unwrap();
            assert!(matches!(
                nav.current().node(),
                DataRef::Value(Value::Int(9))
            ));
            let visit = DataRef::root(source)
                .visits(&cancel)
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(visit.path.0.len(), path_len);
        }
        let mut nav = Navigator::new(&single, &cancel).unwrap();
        assert!(matches!(
            nav.jump(&LogicalPath(vec![PathMember::Index(0); 64]), &cancel),
            Err(DataError::TooDeep)
        ));
        assert_eq!(Cell::Key("abcdefghijklmnopqrstuvwxyz").summary(5), "abcd…");
    }
    #[test]
    fn empty_record_tables_and_mixed_missing_rows_keep_enter_and_back_semantics() {
        let cancel = no_cancel();
        let empty = input(Payload::ListStream(vec![Value::Record(vec![]); 4]));
        let mut nav = Navigator::new(&empty, &cancel).unwrap();
        assert!(nav.current().is_table());
        assert_eq!(nav.current().column_count(), 1);
        assert_eq!(nav.current().column_name(0), Some("record"));
        assert_eq!(nav.current().column_key(0), None);
        assert!(
            matches!(nav.current().cell(2,0),Some(Cell::Data(DataRef::Value(Value::Record(entries)))) if entries.is_empty())
        );
        nav.current_mut().cursor = Cursor {
            row: 2,
            column: 0,
            top: 2,
            left: 0,
        };
        assert!(nav.enter(2, 0, &cancel).unwrap());
        assert_eq!(nav.current().path, LogicalPath(vec![PathMember::Index(2)]));
        assert_eq!(nav.current().row_count(), 0);
        assert!(nav.back());
        assert_eq!(nav.current().cursor.top, 2);
        let mixed = input(Payload::ListStream(vec![
            Value::Record(vec![]),
            Value::Record(vec![("x".into(), Value::Nothing)]),
        ]));
        let mut nav = Navigator::new(&mixed, &cancel).unwrap();
        assert_eq!(nav.current().column_key(0), Some("x"));
        assert!(matches!(nav.current().cell(0, 0), Some(Cell::Missing)));
        assert!(!nav.enter(0, 0, &cancel).unwrap());
        assert!(nav.enter_row(0, &cancel).unwrap());
        assert_eq!(nav.current().row_count(), 0);
        assert!(nav.back());
        assert_eq!(nav.current().cursor.column, 0);
    }
    #[test]
    fn search_is_lazy_and_cancellable_between_visits() {
        let values: Vec<_> = (0..100_000).map(Value::Int).collect();
        let cancel = no_cancel();
        let mut visits = DataRef::ListStream(&values).visits(&cancel);
        for i in 0..10 {
            let visit = visits.next().unwrap().unwrap();
            assert_eq!(visit.path.0, [BorrowedMember::Index(i)]);
            assert!(visits.stack.len() <= 2);
        }
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(visits.next(), Some(Err(DataError::Cancelled))));
        assert!(visits.next().is_none());
    }
}
