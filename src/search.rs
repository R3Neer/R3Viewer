//! One coalescing search worker; bounded result channel and interruptible chunks.
use crate::data::LogicalPath;
use std::{
    ops::Range,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
const CHUNK: usize = 64 * 1024;
pub const QUERY_LIMIT: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}
#[derive(Clone, Debug)]
pub struct SearchHit {
    pub visit_index: usize,
    pub path: Option<LogicalPath>,
    pub key: bool,
    pub range: Range<usize>,
}
#[derive(Clone, Debug)]
pub struct FileSearchHit {
    pub visit_index: usize,
    pub path: crate::data::FilePath,
    /// Safe cooked/descriptor byte range. Never a raw file coordinate.
    pub range: Range<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YamlSearchDomain {
    ScalarDecoded,
    TagUri,
    AliasDescriptor,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YamlSearchHit {
    pub generation: u64,
    pub visit_index: usize,
    pub node: crate::structure_model::NodeId,
    pub domain: crate::yaml_view::Domain,
    pub search_domain: YamlSearchDomain,
    pub original_range: crate::yaml_view::HitRange,
    pub range: crate::yaml_view::HitRange,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XmlSearchHit {
    pub generation: u64,
    pub visit_index: usize,
    pub node: crate::structure_model::NodeId,
    pub domain: crate::xml_view::Domain,
    pub original_range: crate::xml_view::HitRange,
    pub range: crate::xml_view::HitRange,
}
#[derive(Debug)]
pub enum SearchOutcome {
    Found(SearchHit),
    FileFound(FileSearchHit),
    YamlFound(YamlSearchHit),
    XmlFound(XmlSearchHit),
    SourceFound {
        original: Range<usize>,
        visible: Range<usize>,
    },
    NotFound,
    Cancelled,
    Error(String),
}
struct Job {
    generation: u64,
    cancel: Arc<AtomicBool>,
    run: Box<dyn FnOnce(&AtomicBool) -> SearchOutcome + Send>,
}
#[derive(Default)]
struct State {
    pending: Option<Job>,
    active: Option<Arc<AtomicBool>>,
    shutdown: bool,
}
pub struct SearchWorker {
    state: Arc<(Mutex<State>, Condvar)>,
    results: Option<mpsc::Receiver<(u64, SearchOutcome)>>,
    thread: Option<thread::JoinHandle<()>>,
    generation: u64,
}
impl Default for SearchWorker {
    fn default() -> Self {
        Self::new()
    }
}
impl SearchWorker {
    pub fn new() -> Self {
        let state = Arc::new((Mutex::new(State::default()), Condvar::new()));
        let thread_state = state.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            loop {
                let job = {
                    let (lock, cv) = &*thread_state;
                    let mut state = lock.lock().unwrap();
                    while state.pending.is_none() && !state.shutdown {
                        state = cv.wait(state).unwrap();
                    }
                    if state.shutdown {
                        break;
                    }
                    let job = state.pending.take().unwrap();
                    state.active = Some(job.cancel.clone());
                    job
                };
                let outcome = (job.run)(&job.cancel);
                let mut result = (job.generation, outcome);
                while !job.cancel.load(Ordering::Relaxed) {
                    match tx.try_send(result) {
                        Ok(()) => break,
                        Err(mpsc::TrySendError::Disconnected(_)) => return,
                        Err(mpsc::TrySendError::Full(old)) => {
                            result = old;
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                }
            }
        });
        Self {
            state,
            results: Some(rx),
            thread: Some(worker),
            generation: 0,
        }
    }
    pub fn submit(
        &mut self,
        run: impl FnOnce(&AtomicBool) -> SearchOutcome + Send + 'static,
    ) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        let (lock, cv) = &*self.state;
        let mut state = lock.lock().unwrap();
        if let Some(token) = &state.active {
            token.store(true, Ordering::Relaxed);
        }
        if let Some(job) = state.pending.take() {
            job.cancel.store(true, Ordering::Relaxed);
        }
        state.pending = Some(Job {
            generation: self.generation,
            cancel: Arc::new(AtomicBool::new(false)),
            run: Box::new(run),
        });
        cv.notify_one();
        self.generation
    }
    pub fn poll(&self) -> Option<SearchOutcome> {
        let receiver = self.results.as_ref()?;
        while let Ok((generation, outcome)) = receiver.try_recv() {
            if generation == self.generation {
                return Some(outcome);
            }
        }
        None
    }
    pub fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        let mut state = self.state.0.lock().unwrap();
        if let Some(token) = &state.active {
            token.store(true, Ordering::Relaxed);
        }
        if let Some(job) = state.pending.take() {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }
}
impl Drop for SearchWorker {
    fn drop(&mut self) {
        self.cancel();
        self.state.0.lock().unwrap().shutdown = true;
        self.state.1.notify_one();
        self.results.take(); // Close receiver before joining a producer.
        if let Some(worker) = self.thread.take() {
            let _ = worker.join();
        }
    }
}
fn floor_boundary(s: &str, mut offset: usize) -> usize {
    offset = offset.min(s.len());
    while !s.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
fn ceil_boundary(s: &str, mut offset: usize) -> usize {
    offset = offset.min(s.len());
    while !s.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}
fn segment(
    text: &str,
    query: &str,
    range: Range<usize>,
    direction: Direction,
    cancelled: &AtomicBool,
) -> Result<Option<Range<usize>>, ()> {
    let mut begin = range.start;
    let mut end = range.end;
    while begin < end {
        if cancelled.load(Ordering::Relaxed) {
            return Err(());
        }
        match direction {
            Direction::Forward => {
                let next = ceil_boundary(text, begin.saturating_add(CHUNK).min(end));
                let extended = ceil_boundary(
                    text,
                    next.saturating_add(query.len().saturating_sub(1)).min(end),
                );
                if let Some(found) = text[begin..extended].find(query) {
                    let offset = begin + found;
                    return Ok(Some(offset..offset + query.len()));
                }
                begin = next;
            }
            Direction::Backward => {
                let next = floor_boundary(text, end.saturating_sub(CHUNK).max(begin));
                let extended = floor_boundary(
                    text,
                    next.saturating_sub(query.len().saturating_sub(1))
                        .max(begin),
                );
                if let Some(found) = text[extended..end].rfind(query) {
                    let offset = extended + found;
                    return Ok(Some(offset..offset + query.len()));
                }
                end = next;
            }
        }
    }
    Ok(None)
}
/// Matches are non-overlapping on n/N; wrap once at the document boundary.
pub fn find_visible(
    text: &str,
    query: &str,
    start: usize,
    direction: Direction,
    cancelled: &AtomicBool,
) -> Result<Option<Range<usize>>, String> {
    if query.is_empty() {
        return Ok(None);
    }
    if query.len() > QUERY_LIMIT {
        return Err(format!("consulta excede{QUERY_LIMIT}bytes"));
    }
    let start = floor_boundary(text, start);
    let ranges = match direction {
        Direction::Forward => [start..text.len(), 0..start],
        Direction::Backward => [0..start, start..text.len()],
    };
    for range in ranges {
        match segment(text, query, range, direction, cancelled) {
            Ok(Some(found)) => return Ok(Some(found)),
            Ok(None) => (),
            Err(()) => return Err("búsqueda cancelada".into()),
        }
    }
    Ok(None)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunk_crossing_unicode_both_directions_and_wrap() {
        let text = format!("{}界🦊xyz{}", "a".repeat(CHUNK - 1), "b".repeat(CHUNK));
        let query = "界🦊xyz";
        let expected = CHUNK - 1..CHUNK - 1 + query.len();
        let cancel = AtomicBool::new(false);
        assert_eq!(
            find_visible(&text, query, 0, Direction::Forward, &cancel).unwrap(),
            Some(expected.clone())
        );
        assert_eq!(
            find_visible(&text, query, text.len(), Direction::Backward, &cancel).unwrap(),
            Some(expected.clone())
        );
        assert_eq!(
            find_visible(&text, query, expected.end, Direction::Forward, &cancel).unwrap(),
            Some(expected)
        );
        assert_eq!(
            find_visible("ab abab", "ab", 2, Direction::Backward, &cancel).unwrap(),
            Some(0..2)
        );
    }
    #[test]
    fn cancelled_and_overbudget() {
        assert!(
            find_visible(
                &"x".repeat(CHUNK * 2),
                "none",
                0,
                Direction::Forward,
                &AtomicBool::new(true)
            )
            .is_err()
        );
        assert!(
            find_visible(
                "x",
                &"x".repeat(QUERY_LIMIT + 1),
                0,
                Direction::Forward,
                &AtomicBool::new(false)
            )
            .is_err()
        );
    }
    #[test]
    fn superseded_job_never_applies_and_drop_releases_worker() {
        let mut worker = SearchWorker::new();
        worker.submit(|cancel| {
            while !cancel.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(1));
            }
            SearchOutcome::Error("stale".into())
        });
        worker.submit(|_| SearchOutcome::NotFound);
        let started = std::time::Instant::now();
        loop {
            if let Some(result) = worker.poll() {
                assert!(matches!(result, SearchOutcome::NotFound));
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(1));
        }
        worker.submit(|cancel| {
            while !cancel.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(1));
            }
            SearchOutcome::Cancelled
        });
        drop(worker);
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}

/// Whole-data literal search in DFS visitation order, including collapsed keys.
/// Only one tentative result is retained; n/N wrap once and do not overlap.
pub fn find_data(
    input: &r3viewer_wire::TypedInput,
    query: &str,
    previous: Option<&SearchHit>,
    direction: Direction,
    cancelled: &AtomicBool,
) -> SearchOutcome {
    use crate::{
        data::{BorrowedPath, DataRef, VisitItem},
        inspect_text::{copy_text, primitive_text},
        presentation::SafeTextModel,
    };
    if query.is_empty() {
        return SearchOutcome::NotFound;
    }
    if query.len() > QUERY_LIMIT {
        return SearchOutcome::Error(format!("consulta excede{QUERY_LIMIT}bytes"));
    }
    for pass in 0..if previous.is_some() { 2 } else { 1 } {
        let mut tentative: Option<(BorrowedPath<'_>, bool, usize, Range<usize>)> = None;
        for (index, visit) in DataRef::root(input).visits(cancelled).enumerate() {
            let visit = match visit {
                Ok(x) => x,
                Err(e) => {
                    return if cancelled.load(Ordering::Relaxed) {
                        SearchOutcome::Cancelled
                    } else {
                        SearchOutcome::Error(e.to_string())
                    };
                }
            };
            if let Some(last) = previous {
                let relevant = match (direction, pass) {
                    (Direction::Forward, 0) | (Direction::Backward, 1) => index >= last.visit_index,
                    _ => index <= last.visit_index,
                };
                if !relevant {
                    continue;
                }
            }
            let (raw, key) = match visit.item {
                VisitItem::Key(key) => {
                    // Key text may be large: share the same cancelable copying path.
                    match copy_text(key, cancelled) {
                        Ok(s) => (s, true),
                        Err(e) => return SearchOutcome::Error(e),
                    }
                }
                VisitItem::Primitive(node) => match primitive_text(node, cancelled) {
                    Ok(s) => (s, false),
                    Err(e) => return SearchOutcome::Error(e),
                },
            };
            let safe = match SafeTextModel::new(raw, cancelled) {
                Ok(x) => x,
                Err(e) => {
                    return SearchOutcome::Error(format!("no se puede preparar búsqueda: {e:?}"));
                }
            };
            let mut range = 0..safe.visible().len();
            if let Some(last) = previous.filter(|last| index == last.visit_index) {
                match (direction, pass) {
                    (Direction::Forward, 0) => range.start = last.range.end.min(range.end),
                    (Direction::Backward, 0) => range.end = last.range.start.min(range.end),
                    (Direction::Forward, _) => range.end = last.range.end.min(range.end),
                    (Direction::Backward, _) => range.start = last.range.start.min(range.end),
                }
            }
            match segment(safe.visible(), query, range, direction, cancelled) {
                Err(()) => return SearchOutcome::Cancelled,
                Ok(None) => (),
                Ok(Some(range)) => {
                    if direction == Direction::Forward {
                        return SearchOutcome::Found(SearchHit {
                            path: Some(visit.path.to_logical_path()),
                            key,
                            visit_index: index,
                            range,
                        });
                    }
                    tentative = Some((visit.path, key, index, range));
                }
            }
        }
        if let Some((path, key, visit_index, range)) = tentative {
            return SearchOutcome::Found(SearchHit {
                path: Some(path.to_logical_path()),
                key,
                visit_index,
                range,
            });
        }
    }
    SearchOutcome::NotFound
}

#[cfg(test)]
mod typed_search_tests {
    use super::*;
    use r3viewer_wire::{Origin, Payload, Start, TypedInput, VERSION, Value};
    fn input(v: Value) -> TypedInput {
        TypedInput {
            start: Start {
                version: VERSION,
                cwd: "fixture".into(),
                origin: Origin::SingleValue,
            },
            payload: Payload::SingleValue(v),
        }
    }
    #[test]
    fn collapsed_full_leaves_order_keys_and_wrap_single_match() {
        let data = input(Value::Record(vec![(
            "needle-key".into(),
            Value::Record(vec![(
                "z".into(),
                Value::String(format!("{}needle", "x".repeat(100000))),
            )]),
        )]));
        let cancel = AtomicBool::new(false);
        let SearchOutcome::Found(key) =
            find_data(&data, "needle", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert!(key.key);
        let SearchOutcome::Found(value) =
            find_data(&data, "needle", Some(&key), Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert!(!value.key);
        assert_eq!(value.range, 100000..100006);
        let SearchOutcome::Found(wrapped) =
            find_data(&data, "needle", Some(&value), Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert!(wrapped.key);
        let SearchOutcome::Found(previous) =
            find_data(&data, "needle", Some(&key), Direction::Backward, &cancel)
        else {
            panic!()
        };
        assert_eq!(previous.range, value.range);
        let only = input(Value::String("unique".into()));
        let SearchOutcome::Found(hit) =
            find_data(&only, "unique", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        for direction in [Direction::Forward, Direction::Backward] {
            let SearchOutcome::Found(wrapped) =
                find_data(&only, "unique", Some(&hit), direction, &cancel)
            else {
                panic!()
            };
            assert_eq!(wrapped.range, hit.range);
        }
    }
    #[test]
    fn visible_placeholder_search_retains_source_identity() {
        let input = input(Value::Record(vec![(
            "key".into(),
            Value::String("\x1bfoo ␛foo".into()),
        )]));
        let cancel = AtomicBool::new(false);
        let SearchOutcome::Found(first) =
            find_data(&input, "␛foo", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        let SearchOutcome::Found(next) =
            find_data(&input, "␛foo", Some(&first), Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert_ne!(first.range, next.range);
        assert_eq!(first.path, next.path);
    }
}

pub fn find_structure(
    structure: &crate::structure_model::Structure,
    query: &str,
    previous: Option<&FileSearchHit>,
    direction: Direction,
    cancel: &AtomicBool,
) -> SearchOutcome {
    use crate::{data::FileVisits, inspect_text::file_leaf_text, presentation::SafeTextModel};
    if query.is_empty() {
        return SearchOutcome::NotFound;
    }
    if query.len() > QUERY_LIMIT {
        return SearchOutcome::Error(format!("consulta excede {QUERY_LIMIT} bytes"));
    }
    for pass in 0..if previous.is_some() { 2 } else { 1 } {
        let mut tentative = None;
        for (index, visit) in FileVisits::new(structure, cancel).enumerate() {
            if let Some(last) = previous {
                let relevant = match (direction, pass) {
                    (Direction::Forward, 0) | (Direction::Backward, 1) => index >= last.visit_index,
                    _ => index <= last.visit_index,
                };
                if !relevant {
                    continue;
                }
            }
            let visit = match visit {
                Ok(visit) => visit,
                Err(error) => {
                    return if cancel.load(Ordering::Relaxed) {
                        SearchOutcome::Cancelled
                    } else {
                        SearchOutcome::Error(error.to_string())
                    };
                }
            };
            let raw = match file_leaf_text(visit.leaf, cancel) {
                Ok(raw) => raw,
                Err(error) => {
                    return if cancel.load(Ordering::Relaxed) {
                        SearchOutcome::Cancelled
                    } else {
                        SearchOutcome::Error(error)
                    };
                }
            };
            let safe = match SafeTextModel::new(raw, cancel) {
                Ok(safe) => safe,
                Err(error) => {
                    return if cancel.load(Ordering::Relaxed) {
                        SearchOutcome::Cancelled
                    } else {
                        SearchOutcome::Error(format!("no se puede preparar búsqueda: {error:?}"))
                    };
                }
            };
            let mut range = 0..safe.original().len();
            if let Some(last) = previous.filter(|last| index == last.visit_index) {
                let original = safe.source_range(last.range.clone());
                match (direction, pass) {
                    (Direction::Forward, 0) => range.start = original.end.min(range.end),
                    (Direction::Backward, 0) => range.end = original.start.min(range.end),
                    (Direction::Forward, _) => range.end = original.end.min(range.end),
                    (Direction::Backward, _) => range.start = original.start.min(range.end),
                }
            }
            // Match decoded content, then translate only the highlight to safe display.
            // A placeholder for ESC must not impersonate a literal control-picture glyph.
            match segment(safe.original(), query, range, direction, cancel) {
                Err(()) => return SearchOutcome::Cancelled,
                Ok(None) => (),
                Ok(Some(range)) => {
                    let hit = FileSearchHit {
                        visit_index: index,
                        path: visit.path,
                        range: safe.visible_range(range),
                    };
                    if direction == Direction::Forward {
                        return SearchOutcome::FileFound(hit);
                    }
                    tentative = Some(hit);
                }
            }
        }
        if let Some(hit) = tentative {
            return SearchOutcome::FileFound(hit);
        }
    }
    SearchOutcome::NotFound
}

#[cfg(test)]
mod file_search_tests {
    use super::*;
    #[test]
    fn file_keys_and_cooked_leaves_keep_separate_domains_and_wrap_in_both_directions() {
        let structure = crate::data::file_fixture::sample();
        let cancel = AtomicBool::new(false);
        let SearchOutcome::FileFound(key) =
            find_structure(&structure, "a.b", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert_eq!(key.path.0[0].part, crate::data::FilePart::Key);
        let SearchOutcome::FileFound(cooked) =
            find_structure(&structure, "界\x1b", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert_eq!(cooked.path.0[0].part, crate::data::FilePart::Value);
        assert_eq!(cooked.range, 0..6);
        let raw = structure
            .root()
            .entry(0)
            .unwrap()
            .value()
            .source_location()
            .unwrap()
            .range();
        assert_ne!(raw, cooked.range);
        for direction in [Direction::Forward, Direction::Backward] {
            let SearchOutcome::FileFound(wrapped) =
                find_structure(&structure, "界\x1b", Some(&cooked), direction, &cancel)
            else {
                panic!()
            };
            assert_eq!(wrapped.path, cooked.path);
            assert_eq!(wrapped.range, cooked.range);
        }
        assert!(matches!(
            find_structure(&structure, "\\u754c", None, Direction::Forward, &cancel),
            SearchOutcome::NotFound
        ));
        assert!(matches!(
            find_structure(&structure, "0xF", None, Direction::Forward, &cancel),
            SearchOutcome::FileFound(_)
        ));
        assert!(matches!(
            find_structure(
                &structure,
                "x",
                None,
                Direction::Forward,
                &AtomicBool::new(true)
            ),
            SearchOutcome::Cancelled
        ));
    }
    #[test]
    fn interpreted_search_reads_complete_long_cooked_leaf_without_summary_truncation() {
        use crate::structure_model::*;
        use std::sync::Arc;
        let source = Arc::new("\"large\"".to_owned());
        let mut builder = StructureBuilder::new(source, 7).unwrap();
        let cooked = builder
            .append_cooked(format!("{}needle\x1b", "界".repeat(100_000)).as_bytes())
            .unwrap();
        let root = builder
            .push_node(NodeSpec {
                kind: StructureKind::String,
                text: Some(TextRef::Cooked {
                    start: cooked.start,
                    end: cooked.end,
                }),
                location: Some(SourceSpan { start: 0, end: 7 }),
            })
            .unwrap();
        let cancel = AtomicBool::new(false);
        let structure = builder.finish(root, 0, 0, &cancel).unwrap();
        let SearchOutcome::FileFound(hit) =
            find_structure(&structure, "needle\x1b", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert_eq!(hit.range, 300_000..300_009);
        assert!(hit.path.0.is_empty());
        for direction in [Direction::Forward, Direction::Backward] {
            let SearchOutcome::FileFound(next) =
                find_structure(&structure, "needle\x1b", Some(&hit), direction, &cancel)
            else {
                panic!()
            };
            assert_eq!(next.range, hit.range);
        }
    }
}

#[cfg(test)]
mod file_search_control_identity_tests {
    use super::*;
    #[test]
    fn decoded_controls_do_not_match_literal_display_glyphs_or_escape_labels() {
        let cancel = AtomicBool::new(false);
        let (structure, _, _) = crate::json_worker::project(
            std::sync::Arc::new(r#"{"escaped":"\u001b\u009b","glyph":"␛\\u{009b}"}"#.to_owned()),
            0,
            &cancel,
        )
        .unwrap();
        for query in ["␛", "\\u{009b}"] {
            let SearchOutcome::FileFound(first) =
                find_structure(&structure, query, None, Direction::Forward, &cancel)
            else {
                panic!("literal glyph not found");
            };
            assert_eq!(first.path.0[0].ordinal, 1);
            for direction in [Direction::Forward, Direction::Backward] {
                let SearchOutcome::FileFound(next) =
                    find_structure(&structure, query, Some(&first), direction, &cancel)
                else {
                    panic!("literal glyph wrap not found");
                };
                assert_eq!(next.path, first.path);
                assert_eq!(next.range, first.range);
            }
        }
        let SearchOutcome::FileFound(control) =
            find_structure(&structure, "\x1b", None, Direction::Forward, &cancel)
        else {
            panic!("decoded ESC not found");
        };
        assert_eq!(control.path.0[0].ordinal, 0);
        assert_eq!(control.range, 0..3);
    }
}

/// Decoded and metadata domains use the same borrowed fragments as the pane.
/// No range is interpreted as a source coordinate and Alias never follows DFS.
pub fn find_yaml(
    structure: &crate::structure_model::Structure,
    query: &str,
    previous: Option<&YamlSearchHit>,
    direction: Direction,
    budget: &crate::yaml_view::Budget,
    cancel: &AtomicBool,
) -> SearchOutcome {
    use crate::yaml_view::{Domain, Error, HitRange, Text, Visits};
    let search = || -> Result<Option<YamlSearchHit>, Error> {
        if query.is_empty() {
            return Ok(None);
        }
        if query.len() > QUERY_LIMIT {
            return Err(Error::Budget);
        }
        // KMP table + cloned query/editor strings, fixed DFS/fragments and staging.
        let _charge = budget.charge(QUERY_LIMIT * std::mem::size_of::<usize>() + 65536)?;
        let mut prefix = Vec::new();
        prefix
            .try_reserve_exact(query.len())
            .map_err(|_| Error::Budget)?;
        if prefix.capacity() > query.len() {
            return Err(Error::Budget);
        }
        prefix.resize(query.len(), 0);
        let needle = query.as_bytes();
        let mut k = 0;
        for i in 1..needle.len() {
            if i.is_multiple_of(1024) && cancel.load(Ordering::Relaxed) {
                return Err(Error::Cancelled);
            }
            while k > 0 && needle[k] != needle[i] {
                k = prefix[k - 1]
            }
            if needle[k] == needle[i] {
                k += 1
            }
            prefix[i] = k
        }
        for pass in 0..if previous.is_some() { 2 } else { 1 } {
            let mut tentative = None;
            let mut index = 0;
            for node in Visits::new(structure, cancel) {
                let node = node?;
                let reference = structure.node(node).ok_or(Error::Invalid)?;
                for domain in [Domain::Metadata, Domain::Decoded] {
                    if domain == Domain::Decoded && reference.decoded().is_none() {
                        continue;
                    }
                    let text = Text::new(structure, node, domain)?;
                    for (unit_index, unit) in text.search_ranges().into_iter().enumerate() {
                        let Some(unit) = unit else { continue };
                        let search_domain = match (domain, unit_index) {
                            (Domain::Decoded, _) => YamlSearchDomain::ScalarDecoded,
                            (Domain::Metadata, 0) => YamlSearchDomain::TagUri,
                            (Domain::Metadata, _) => YamlSearchDomain::AliasDescriptor,
                        };
                        let visit_index = index;
                        index += 1;
                        if let Some(last) = previous {
                            let relevant = match (direction, pass) {
                                (Direction::Forward, 0) | (Direction::Backward, 1) => {
                                    visit_index >= last.visit_index
                                }
                                _ => visit_index <= last.visit_index,
                            };
                            if !relevant {
                                continue;
                            }
                        }
                        let mut from = unit.start;
                        let mut to = unit.end;
                        if let Some(last) = previous.filter(|last| last.visit_index == visit_index)
                        {
                            if last.generation != structure.generation()
                                || last.node != node
                                || last.domain != domain
                                || last.search_domain != search_domain
                                || last.original_range.end > unit.end - unit.start
                                || last.original_range.start > last.original_range.end
                            {
                                return Err(Error::Invalid);
                            }
                            match (direction, pass) {
                                (Direction::Forward, 0) => {
                                    from = unit.start + last.original_range.end
                                }
                                (Direction::Backward, 0) => {
                                    to = unit.start + last.original_range.start
                                }
                                (Direction::Forward, _) => {
                                    to = unit.start + last.original_range.end
                                }
                                (Direction::Backward, _) => {
                                    from = unit.start + last.original_range.start
                                }
                            }
                        }
                        let mut matched = 0;
                        for (offset, byte) in text.raw_bytes().enumerate() {
                            if offset.is_multiple_of(1024) {
                                crate::yaml_view::check_search(cancel)?;
                            }
                            if offset < from {
                                continue;
                            }
                            if offset >= to {
                                break;
                            }
                            while matched > 0 && needle[matched] != byte {
                                matched = prefix[matched - 1]
                            }
                            if needle[matched] == byte {
                                matched += 1
                            }
                            if matched == needle.len() {
                                let range = HitRange {
                                    start: offset + 1 - needle.len(),
                                    end: offset + 1,
                                };
                                let hit = YamlSearchHit {
                                    generation: structure.generation(),
                                    visit_index,
                                    node,
                                    domain,
                                    search_domain,
                                    original_range: HitRange {
                                        start: range.start - unit.start,
                                        end: range.end - unit.start,
                                    },
                                    // Tentative backward matches retain pane-original bytes.
                                    // Project only the chosen hit, never every prefix match.
                                    range,
                                };
                                if direction == Direction::Forward {
                                    let mut hit = hit;
                                    hit.range = text.visible_range(hit.range, cancel)?;
                                    return Ok(Some(hit));
                                }
                                tentative = Some(hit);
                                matched = prefix[needle.len() - 1];
                            }
                        }
                    }
                }
            }
            if let Some(mut hit) = tentative {
                let text = Text::new(structure, hit.node, hit.domain)?;
                hit.range = text.visible_range(hit.range, cancel)?;
                return Ok(Some(hit));
            }
        }
        Ok(None)
    };
    match search() {
        Ok(Some(hit)) => SearchOutcome::YamlFound(hit),
        Ok(None) => SearchOutcome::NotFound,
        Err(Error::Cancelled) => SearchOutcome::Cancelled,
        Err(error) => SearchOutcome::Error(error.to_string()),
    }
}
pub fn find_xml(
    structure: &crate::structure_model::Structure,
    query: &str,
    previous: Option<&XmlSearchHit>,
    direction: Direction,
    budget: &crate::xml_view::Budget,
    cancel: &AtomicBool,
) -> SearchOutcome {
    use crate::xml_view::{Error, HitRange, Text, Visits};
    let search = || -> Result<Option<XmlSearchHit>, Error> {
        if previous.is_some_and(|last| last.generation != structure.generation()) {
            return Err(Error::Invalid);
        }
        if query.is_empty() {
            return Ok(None);
        }
        if query.len() > QUERY_LIMIT {
            return Err(Error::Budget);
        }
        // KMP table + cloned query/editor strings, fixed DFS/fragments and staging.
        let _charge = budget.charge(QUERY_LIMIT * std::mem::size_of::<usize>() + 65536)?;
        let mut prefix = Vec::new();
        prefix
            .try_reserve_exact(query.len())
            .map_err(|_| Error::Budget)?;
        if prefix.capacity() > query.len() {
            return Err(Error::Budget);
        }
        prefix.resize(query.len(), 0);
        let needle = query.as_bytes();
        let mut k = 0;
        for i in 1..needle.len() {
            if i.is_multiple_of(1024) && cancel.load(Ordering::Relaxed) {
                return Err(Error::Cancelled);
            }
            while k > 0 && needle[k] != needle[i] {
                k = prefix[k - 1]
            }
            if needle[k] == needle[i] {
                k += 1
            }
            prefix[i] = k
        }
        for pass in 0..if previous.is_some() { 2 } else { 1 } {
            let mut tentative = None;
            let mut index = 0;
            for node in Visits::new(structure, cancel) {
                let node = node?;
                for domain in crate::xml_view::SEARCH_DOMAINS {
                    if crate::xml_view::unit(structure, node, domain).is_none() {
                        continue;
                    }
                    let text = Text::new(structure, node, domain)?;
                    for unit in text.search_ranges() {
                        let Some(unit) = unit else { continue };
                        let visit_index = index;
                        index += 1;
                        if let Some(last) = previous {
                            let relevant = match (direction, pass) {
                                (Direction::Forward, 0) | (Direction::Backward, 1) => {
                                    visit_index >= last.visit_index
                                }
                                _ => visit_index <= last.visit_index,
                            };
                            if !relevant {
                                continue;
                            }
                        }
                        let mut from = unit.start;
                        let mut to = unit.end;
                        if let Some(last) = previous.filter(|last| last.visit_index == visit_index)
                        {
                            if last.generation != structure.generation()
                                || last.node != node
                                || last.domain != domain
                                || last.original_range.end > unit.end - unit.start
                                || last.original_range.start > last.original_range.end
                            {
                                return Err(Error::Invalid);
                            }
                            match (direction, pass) {
                                (Direction::Forward, 0) => {
                                    from = unit.start + last.original_range.end
                                }
                                (Direction::Backward, 0) => {
                                    to = unit.start + last.original_range.start
                                }
                                (Direction::Forward, _) => {
                                    to = unit.start + last.original_range.end
                                }
                                (Direction::Backward, _) => {
                                    from = unit.start + last.original_range.start
                                }
                            }
                        }
                        let mut matched = 0;
                        for (offset, byte) in text.raw_bytes().enumerate() {
                            if offset.is_multiple_of(1024) {
                                crate::xml_view::check_search(cancel)?;
                            }
                            if offset < from {
                                continue;
                            }
                            if offset >= to {
                                break;
                            }
                            while matched > 0 && needle[matched] != byte {
                                matched = prefix[matched - 1]
                            }
                            if needle[matched] == byte {
                                matched += 1
                            }
                            if matched == needle.len() {
                                let range = HitRange {
                                    start: offset + 1 - needle.len(),
                                    end: offset + 1,
                                };
                                let hit = XmlSearchHit {
                                    generation: structure.generation(),
                                    visit_index,
                                    node,
                                    domain,
                                    original_range: HitRange {
                                        start: range.start - unit.start,
                                        end: range.end - unit.start,
                                    },
                                    // Tentative backward matches retain pane-original bytes.
                                    // Project only the chosen hit, never every prefix match.
                                    range,
                                };
                                if direction == Direction::Forward {
                                    let mut hit = hit;
                                    hit.range = text.visible_range(hit.range, cancel)?;
                                    return Ok(Some(hit));
                                }
                                tentative = Some(hit);
                                matched = prefix[needle.len() - 1];
                            }
                        }
                    }
                }
            }
            if let Some(mut hit) = tentative {
                let text = Text::new(structure, hit.node, hit.domain)?;
                hit.range = text.visible_range(hit.range, cancel)?;
                return Ok(Some(hit));
            }
        }
        Ok(None)
    };
    match search() {
        Ok(Some(hit)) => SearchOutcome::XmlFound(hit),
        Ok(None) => SearchOutcome::NotFound,
        Err(Error::Cancelled) => SearchOutcome::Cancelled,
        Err(error) => SearchOutcome::Error(error.to_string()),
    }
}

#[cfg(test)]
mod yaml_search_tests {
    use super::*;
    fn model(raw: &str) -> crate::structure_model::Structure {
        crate::yaml_worker::project(Arc::new(raw.to_owned()), 1, &AtomicBool::new(false))
            .unwrap()
            .0
    }
    #[test]
    fn decoded_tagged_numbers_controls_and_metadata_domains_are_complete_and_alias_dfs_is_finite() {
        use crate::yaml_view::{Budget, Domain};
        let structure = model(
            "a: !!int \"\\u0031\\u0032\"\nb: &x \"\\efoo ␛foo\"\nc: *x\nd: !<tag:example:needle> opaque\n",
        );
        let cancel = AtomicBool::new(false);
        let budget = Budget::new(&structure).unwrap();
        let SearchOutcome::YamlFound(number) =
            find_yaml(&structure, "12", None, Direction::Forward, &budget, &cancel)
        else {
            panic!()
        };
        assert_eq!(number.domain, Domain::Decoded);
        assert_eq!(number.range.start, 0);
        assert!(matches!(
            find_yaml(
                &structure,
                "\\u0031",
                None,
                Direction::Forward,
                &budget,
                &cancel
            ),
            SearchOutcome::NotFound
        ));
        let SearchOutcome::YamlFound(first) = find_yaml(
            &structure,
            "␛foo",
            None,
            Direction::Forward,
            &budget,
            &cancel,
        ) else {
            panic!()
        };
        let SearchOutcome::YamlFound(second) = find_yaml(
            &structure,
            "␛foo",
            Some(&first),
            Direction::Forward,
            &budget,
            &cancel,
        ) else {
            panic!()
        };
        assert_eq!(first.node, second.node);
        assert_eq!(first.range, second.range);
        let SearchOutcome::YamlFound(previous) = find_yaml(
            &structure,
            "␛foo",
            Some(&second),
            Direction::Backward,
            &budget,
            &cancel,
        ) else {
            panic!()
        };
        assert_eq!(previous, first);
        let SearchOutcome::YamlFound(tag) = find_yaml(
            &structure,
            "tag:example:needle",
            None,
            Direction::Forward,
            &budget,
            &cancel,
        ) else {
            panic!()
        };
        assert_eq!(tag.domain, Domain::Metadata);
        let cycle = model("&a [*a]");
        let b = Budget::new(&cycle).unwrap();
        assert!(matches!(
            find_yaml(&cycle, "absent", None, Direction::Forward, &b, &cancel),
            SearchOutcome::NotFound
        ));
        assert!(matches!(
            find_yaml(&cycle, "target node", None, Direction::Forward, &b, &cancel),
            SearchOutcome::NotFound
        ));
        assert!(matches!(
            find_yaml(
                &structure,
                "12",
                None,
                Direction::Forward,
                &budget,
                &AtomicBool::new(true)
            ),
            SearchOutcome::Cancelled
        ));
        assert_eq!(budget.used(), 0);
    }
    #[test]
    fn yaml_matches_original_controls_and_only_tag_alias_metadata() {
        let structure = model(
            "a: \"\\e\\u009b\"\nb: \"␛\\\\u{009b}\"\nc: !<tag:example:needle> opaque\nd: &cycle [*cycle]\n",
        );
        let budget = crate::yaml_view::Budget::new(&structure).unwrap();
        let cancel = AtomicBool::new(false);
        let find = |q: &str| find_yaml(&structure, q, None, Direction::Forward, &budget, &cancel);
        let SearchOutcome::YamlFound(control) = find("\x1b") else {
            panic!("ESC missing")
        };
        assert_eq!(
            control.original_range,
            crate::yaml_view::HitRange { start: 0, end: 1 }
        );
        assert_eq!(
            control.range,
            crate::yaml_view::HitRange { start: 0, end: 3 }
        );
        for query in ["␛", "\\u{009b}"] {
            let SearchOutcome::YamlFound(literal) = find(query) else {
                panic!("literal missing")
            };
            assert_ne!(literal.node, control.node);
            for direction in [Direction::Forward, Direction::Backward] {
                let SearchOutcome::YamlFound(next) = find_yaml(
                    &structure,
                    query,
                    Some(&literal),
                    direction,
                    &budget,
                    &cancel,
                ) else {
                    panic!("wrap missing")
                };
                assert_eq!(next, literal);
            }
        }
        for query in [
            "estilo:",
            "target node",
            "documento:",
            "inspector de serialización",
            "Enter sigue",
        ] {
            assert!(
                matches!(find(query), SearchOutcome::NotFound),
                "unexpected metadata {query}"
            );
        }
        assert!(matches!(
            find("tag:example:needle"),
            SearchOutcome::YamlFound(_)
        ));
        assert!(matches!(
            find("alias *cycle -> anchor#"),
            SearchOutcome::YamlFound(_)
        ));
    }
    #[test]
    fn yaml_backward_search_chooses_rightmost_match_then_nonoverlap_previous() {
        let structure = model("\"aaaaa\"");
        let budget = crate::yaml_view::Budget::new(&structure).unwrap();
        let cancel = AtomicBool::new(false);
        let SearchOutcome::YamlFound(last) = find_yaml(
            &structure,
            "aa",
            None,
            Direction::Backward,
            &budget,
            &cancel,
        ) else {
            panic!()
        };
        assert_eq!((last.range.start, last.range.end), (3, 5));
        let SearchOutcome::YamlFound(previous) = find_yaml(
            &structure,
            "aa",
            Some(&last),
            Direction::Backward,
            &budget,
            &cancel,
        ) else {
            panic!()
        };
        assert_eq!((previous.range.start, previous.range.end), (1, 3));
    }
}

#[cfg(test)]
mod xml_search_tests {
    use super::*;
    use crate::xml_view::{Budget, Domain};
    fn model(raw: &str) -> crate::structure_model::Structure {
        crate::xml_worker::project(Arc::new(raw.into()), 4, &AtomicBool::new(false))
            .unwrap()
            .0
    }
    #[test]
    fn original_controls_literal_and_domain_identity_remain_separate() {
        let s = model("<same>\u{009b} \\u{009b}</same>");
        let cancel = AtomicBool::new(false);
        let budget = Budget::new(&s).unwrap();
        let SearchOutcome::XmlFound(control) =
            find_xml(&s, "\u{009b}", None, Direction::Forward, &budget, &cancel)
        else {
            panic!("control")
        };
        assert_eq!(control.domain, Domain::ValueDecoded);
        assert_eq!(control.original_range.range(), 0..2);
        assert_eq!(control.range.range(), 0..8);
        let SearchOutcome::XmlFound(literal) =
            find_xml(&s, "\\u{009b}", None, Direction::Forward, &budget, &cancel)
        else {
            panic!("literal")
        };
        assert_eq!(literal.original_range.range(), 3..11);
        assert_eq!(literal.range.range(), 9..17);
        let SearchOutcome::XmlFound(qname) =
            find_xml(&s, "same", None, Direction::Forward, &budget, &cancel)
        else {
            panic!("qname")
        };
        assert_eq!(qname.domain, Domain::QName);
        let SearchOutcome::XmlFound(local) = find_xml(
            &s,
            "same",
            Some(&qname),
            Direction::Forward,
            &budget,
            &cancel,
        ) else {
            panic!("local")
        };
        assert_eq!(local.domain, Domain::LocalName);
        assert_eq!(local.node, qname.node);
        assert!(matches!(
            find_xml(&s, "inspector", None, Direction::Forward, &budget, &cancel),
            SearchOutcome::NotFound
        ));
        let mut stale = local;
        stale.generation += 1;
        assert!(matches!(
            find_xml(
                &s,
                "same",
                Some(&stale),
                Direction::Forward,
                &budget,
                &cancel
            ),
            SearchOutcome::Error(_)
        ));
        assert_eq!(budget.used(), 0);
    }
    #[test]
    fn backward_selects_only_final_projection_and_does_not_cross_nodes() {
        let s = model(&format!("<r>{}<a/>b</r>", "a".repeat(200_000)));
        let budget = Budget::new(&s).unwrap();
        let cancel = AtomicBool::new(false);
        let SearchOutcome::XmlFound(last) =
            find_xml(&s, "a", None, Direction::Backward, &budget, &cancel)
        else {
            panic!()
        };
        // QName/local name of child a follows the large text physically. Search text directly by resuming before child.
        let mut previous = last;
        while previous.domain != Domain::ValueDecoded {
            let SearchOutcome::XmlFound(hit) = find_xml(
                &s,
                "a",
                Some(&previous),
                Direction::Backward,
                &budget,
                &cancel,
            ) else {
                panic!()
            };
            previous = hit;
        }
        assert_eq!(previous.original_range.range(), 199_999..200_000);
        assert!(matches!(
            find_xml(&s, "ab", None, Direction::Forward, &budget, &cancel),
            SearchOutcome::NotFound
        ));
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            find_xml(&s, "a", None, Direction::Forward, &budget, &cancel),
            SearchOutcome::Cancelled
        ));
        assert_eq!(budget.used(), 0);
    }
}

/// Source identity remains original UTF8, while only presentation coordinates reach UI.
pub fn find_source(
    source: &crate::presentation::SafeTextModel,
    query: &str,
    previous: Option<&Range<usize>>,
    direction: Direction,
    cancel: &AtomicBool,
) -> SearchOutcome {
    let text = source.original();
    if previous.is_some_and(|r| {
        r.start > r.end
            || r.end > text.len()
            || !text.is_char_boundary(r.start)
            || !text.is_char_boundary(r.end)
    }) {
        return SearchOutcome::Error("rango Fuente original no válido".into());
    }
    let offset = previous.map_or(
        if direction == Direction::Forward {
            0
        } else {
            text.len()
        },
        |r| {
            if direction == Direction::Forward {
                r.end
            } else {
                r.start
            }
        },
    );
    match find_visible(text, query, offset, direction, cancel) {
        Ok(Some(original)) => SearchOutcome::SourceFound {
            visible: source.visible_range(original.clone()),
            original,
        },
        Ok(None) => SearchOutcome::NotFound,
        Err(_) if cancel.load(Ordering::Relaxed) => SearchOutcome::Cancelled,
        Err(e) => SearchOutcome::Error(e),
    }
}
#[cfg(test)]
mod source_original_tests {
    use super::*;
    #[test]
    fn source_matches_original_not_control_placeholders_and_preserves_raw_resume() {
        let cancel = AtomicBool::new(false);
        let source = crate::presentation::SafeTextModel::new(
            "\x1b␛\u{009b} \\u{009b}\r\n\t界e\u{301}end".into(),
            &cancel,
        )
        .unwrap();
        let get = |q: &str, previous: Option<&Range<usize>>, dir| {
            let SearchOutcome::SourceFound { original, visible } =
                find_source(&source, q, previous, dir, &cancel)
            else {
                panic!("{q:?}")
            };
            (original, visible)
        };
        let (esc, shown) = get("\x1b", None, Direction::Forward);
        assert_eq!(esc, 0..1);
        assert_eq!(shown, 0..3);
        let (glyph, _) = get("␛", None, Direction::Forward);
        assert_eq!(glyph, 1..4);
        let (c1, _) = get("\u{009b}", None, Direction::Forward);
        assert_eq!(c1, 4..6);
        let (literal, _) = get("\\u{009b}", None, Direction::Forward);
        assert_eq!(literal, 7..15);
        assert_eq!(
            get("\\u{009b}", Some(&literal), Direction::Forward).0,
            literal
        );
        assert_eq!(get("\r\n", None, Direction::Forward).0, 15..17);
        assert_eq!(get("\t", None, Direction::Forward).0, 17..18);
        let (last, _) = get("end", None, Direction::Backward);
        assert_eq!(last.end, source.original().len());
        assert_eq!(get("end", Some(&last), Direction::Backward).0, last);
        assert!(matches!(
            find_source(&source, "e", Some(&(5..6)), Direction::Forward, &cancel),
            SearchOutcome::Error(_)
        ));
        assert!(matches!(
            find_source(
                &source,
                "end",
                None,
                Direction::Forward,
                &AtomicBool::new(true)
            ),
            SearchOutcome::Cancelled
        ));
    }
}
