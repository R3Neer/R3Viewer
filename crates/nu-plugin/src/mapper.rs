use std::io;
use std::ops::Bound;
use std::sync::atomic::{AtomicBool, Ordering};

use nu_protocol::{Range, Value as NuValue, ast::PathMember, casing::Casing};
use r3viewer_wire::{self as wire, Value};

pub(crate) fn check_cancel(cancel: &AtomicBool) -> io::Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "R3Viewer encoding cancelled",
        ))
    } else {
        Ok(())
    }
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn bound<T>(bound: Bound<T>) -> wire::Bound<T> {
    match bound {
        Bound::Included(value) => wire::Bound::Included(value),
        Bound::Excluded(value) => wire::Bound::Excluded(value),
        Bound::Unbounded => wire::Bound::Unbounded,
    }
}

/// Budgets apply across all Value frames, with a fresh scalar allocation
/// budget for each frame. SDK containers and closure captures are never cloned.
pub(crate) struct Mapper<'a> {
    cancel: &'a AtomicBool,
    nodes: usize,
    scalar_bytes: usize,
}
impl<'a> Mapper<'a> {
    pub(crate) fn new(cancel: &'a AtomicBool) -> Self {
        Self {
            cancel,
            nodes: 0,
            scalar_bytes: 0,
        }
    }
    pub(crate) fn value(&mut self, value: &NuValue) -> io::Result<Value> {
        self.scalar_bytes = 0;
        self.map(value, 1)
    }
    fn charge_bytes(&mut self, length: usize) -> io::Result<()> {
        check_cancel(self.cancel)?;
        self.scalar_bytes = self
            .scalar_bytes
            .checked_add(length)
            .filter(|bytes| *bytes <= wire::MAX_FRAME_BYTES)
            .ok_or_else(|| invalid("Value exceeds the frame byte budget"))?;
        Ok(())
    }
    fn text(&mut self, text: &str) -> io::Result<String> {
        self.charge_bytes(text.len())?;
        let text = text.to_owned();
        check_cancel(self.cancel)?;
        Ok(text)
    }
    fn check_children(&self, count: usize) -> io::Result<()> {
        check_cancel(self.cancel)?;
        if count > wire::MAX_VALUE_NODES.saturating_sub(self.nodes) {
            Err(invalid("Value node limit"))
        } else {
            Ok(())
        }
    }
    fn map(&mut self, value: &NuValue, depth: usize) -> io::Result<Value> {
        check_cancel(self.cancel)?;
        if depth > wire::MAX_VALUE_DEPTH {
            return Err(invalid("Value depth limit"));
        }
        self.nodes = self
            .nodes
            .checked_add(1)
            .filter(|nodes| *nodes <= wire::MAX_VALUE_NODES)
            .ok_or_else(|| invalid("Value node limit"))?;
        let result = match value {
            NuValue::Nothing { .. } => Value::Nothing,
            NuValue::Bool { val, .. } => Value::Bool(*val),
            NuValue::Int { val, .. } => Value::Int(*val),
            NuValue::Float { val, .. } => Value::Float(*val),
            NuValue::String { val, .. } => Value::String(self.text(val)?),
            NuValue::Binary { val, .. } => {
                self.charge_bytes(val.len())?;
                Value::Binary(val.to_vec())
            }
            NuValue::Filesize { val, .. } => Value::Filesize(val.get()),
            NuValue::Duration { val, .. } => Value::Duration(*val),
            NuValue::Date { val, .. } => {
                if val.offset().local_minus_utc() % 60 != 0 {
                    return Err(invalid("RFC3339 cannot preserve a sub-minute date offset"));
                }
                let date = val.to_rfc3339();
                self.charge_bytes(date.len())?;
                Value::Date(date)
            }
            NuValue::Range { val, .. } => match **val {
                Range::IntRange(range) => Value::IntRange {
                    start: range.start(),
                    step: range.step(),
                    end: bound(range.end()),
                },
                Range::FloatRange(range) => Value::FloatRange {
                    start: range.start(),
                    step: range.step(),
                    end: bound(range.end()),
                },
            },
            NuValue::Record { val, .. } => {
                self.check_children(val.len())?;
                if depth == wire::MAX_VALUE_DEPTH && !val.is_empty() {
                    return Err(invalid("Value depth limit"));
                }
                let mut entries = Vec::with_capacity(val.len());
                for (key, child) in val.iter() {
                    let key = self.text(key)?;
                    entries.push((key, self.map(child, depth + 1)?));
                }
                Value::Record(entries)
            }
            NuValue::List { vals, .. } => {
                self.check_children(vals.len())?;
                if depth == wire::MAX_VALUE_DEPTH && !vals.is_empty() {
                    return Err(invalid("Value depth limit"));
                }
                let mut children = Vec::with_capacity(vals.len());
                for child in vals.iter() {
                    children.push(self.map(child, depth + 1)?);
                }
                Value::List(children)
            }
            NuValue::CellPath { val, .. } => {
                // Path components aren't semantic Values, but their allocation
                // is still bounded before reserving their output container.
                if val.members.len() > wire::MAX_VALUE_NODES {
                    return Err(invalid("Cell path component limit"));
                }
                self.charge_bytes(
                    val.members
                        .len()
                        .checked_mul(8)
                        .ok_or_else(|| invalid("Cell path size overflow"))?,
                )?;
                let mut members = Vec::with_capacity(val.members.len());
                for member in &val.members {
                    check_cancel(self.cancel)?;
                    members.push(match member {
                        PathMember::String {
                            val,
                            optional,
                            casing,
                            ..
                        } => wire::CellPathMember::String {
                            value: self.text(val)?,
                            optional: *optional,
                            insensitive: *casing == Casing::Insensitive,
                        },
                        PathMember::Int { val, optional, .. } => wire::CellPathMember::Index {
                            value: u64::try_from(*val)
                                .map_err(|_| invalid("Cell path index overflow"))?,
                            optional: *optional,
                        },
                    });
                }
                Value::CellPath(members)
            }
            NuValue::Glob { val, no_expand, .. } => Value::Glob {
                pattern: self.text(val)?,
                no_expand: *no_expand,
            },
            NuValue::Error { error, .. } => {
                check_cancel(self.cancel)?;
                let message = error.to_string();
                self.charge_bytes(message.len())?;
                Value::Error { message }
            }
            NuValue::Closure { .. } => Value::Closure,
            NuValue::Custom { .. } => {
                return Err(invalid(
                    "Nushell custom values are not supported by R3Viewer wire v1",
                ));
            }
        };
        check_cancel(self.cancel)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::{
        BlockId, Record, Span, VarId,
        ast::{CellPath, RangeInclusion},
        engine::Closure,
    };

    #[test]
    fn nested_records_keep_order_and_types() {
        let input = NuValue::record(
            Record::from_iter([
                (
                    "z".into(),
                    NuValue::list(
                        vec![
                            NuValue::nothing(Span::unknown()),
                            NuValue::int(7, Span::unknown()),
                        ],
                        Span::unknown(),
                    ),
                ),
                ("a".into(), NuValue::string("literal", Span::unknown())),
            ]),
            Span::unknown(),
        );
        let cancel = AtomicBool::new(false);
        assert_eq!(
            Mapper::new(&cancel).value(&input).unwrap(),
            Value::Record(vec![
                ("z".into(), Value::List(vec![Value::Nothing, Value::Int(7)])),
                ("a".into(), Value::String("literal".into())),
            ])
        );
    }
    #[test]
    fn ranges_are_descriptors_including_unbounded_and_exclusive() {
        let span = Span::unknown();
        let int = Range::new(
            NuValue::int(2, span),
            NuValue::int(4, span),
            NuValue::nothing(span),
            RangeInclusion::Inclusive,
            span,
        )
        .unwrap();
        let float = Range::new(
            NuValue::float(1.0, span),
            NuValue::float(1.5, span),
            NuValue::float(3.0, span),
            RangeInclusion::RightExclusive,
            span,
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut mapper = Mapper::new(&cancel);
        assert_eq!(
            mapper.value(&NuValue::range(int, span)).unwrap(),
            Value::IntRange {
                start: 2,
                step: 2,
                end: wire::Bound::Unbounded
            }
        );
        assert_eq!(
            mapper.value(&NuValue::range(float, span)).unwrap(),
            Value::FloatRange {
                start: 1.0,
                step: 0.5,
                end: wire::Bound::Excluded(3.0)
            }
        );
    }
    #[test]
    fn cell_path_preserves_flags_and_closure_omits_captures() {
        let span = Span::unknown();
        let path = NuValue::cell_path(
            CellPath {
                members: vec![
                    PathMember::string("name".into(), true, Casing::Insensitive, span),
                    PathMember::int(2, false, span),
                ],
            },
            span,
        );
        let closure = NuValue::closure(
            Closure {
                block_id: BlockId::new(9),
                captures: vec![(VarId::new(8), NuValue::string("private capture", span))],
            },
            span,
        );
        let cancel = AtomicBool::new(false);
        let mut mapper = Mapper::new(&cancel);
        assert_eq!(
            mapper.value(&path).unwrap(),
            Value::CellPath(vec![
                wire::CellPathMember::String {
                    value: "name".into(),
                    optional: true,
                    insensitive: true
                },
                wire::CellPathMember::Index {
                    value: 2,
                    optional: false
                },
            ])
        );
        assert_eq!(mapper.value(&closure).unwrap(), Value::Closure);
    }
    #[test]
    fn depth_nodes_and_cancel_are_checked() {
        let span = Span::unknown();
        let mut nested = NuValue::nothing(span);
        for _ in 0..wire::MAX_VALUE_DEPTH - 1 {
            nested = NuValue::list(vec![nested], span);
        }
        let cancel = AtomicBool::new(false);
        assert!(Mapper::new(&cancel).value(&nested).is_ok());
        nested = NuValue::list(vec![nested], span);
        assert!(Mapper::new(&cancel).value(&nested).is_err());
        let mut mapper = Mapper::new(&cancel);
        mapper.nodes = wire::MAX_VALUE_NODES - 1;
        assert!(mapper.value(&NuValue::nothing(span)).is_ok());
        assert!(mapper.value(&NuValue::nothing(span)).is_err());
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            Mapper::new(&cancel)
                .value(&NuValue::string("uncloned", span))
                .unwrap_err()
                .kind(),
            io::ErrorKind::Interrupted
        );
    }
    #[test]
    fn scalar_types_preserve_offsets_units_and_float_bits() {
        let span = Span::unknown();
        let cancel = AtomicBool::new(false);
        let mut mapper = Mapper::new(&cancel);
        let date = NuValue::date("2026-10-04T13:14:15.123456789+02:30".parse().unwrap(), span);
        assert_eq!(
            mapper.value(&date).unwrap(),
            Value::Date("2026-10-04T13:14:15.123456789+02:30".into())
        );
        assert_eq!(
            mapper
                .value(&NuValue::filesize(
                    nu_protocol::Filesize::new(i64::MIN),
                    span
                ))
                .unwrap(),
            Value::Filesize(i64::MIN)
        );
        assert_eq!(
            mapper.value(&NuValue::duration(i64::MAX, span)).unwrap(),
            Value::Duration(i64::MAX)
        );
        assert_eq!(
            mapper.value(&NuValue::glob("*.rs", true, span)).unwrap(),
            Value::Glob {
                pattern: "*.rs".into(),
                no_expand: true
            }
        );
        for bits in [
            (-0.0f64).to_bits(),
            f64::INFINITY.to_bits(),
            0x7ff8_0000_0000_1234,
        ] {
            let Value::Float(mapped) = mapper
                .value(&NuValue::float(f64::from_bits(bits), span))
                .unwrap()
            else {
                panic!("float type changed")
            };
            assert_eq!(mapped.to_bits(), bits);
        }
    }
}
