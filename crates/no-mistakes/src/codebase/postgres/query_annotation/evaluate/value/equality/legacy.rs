use super::Value;
use crate::codebase::postgres::query_annotation::Function;
use std::{path::PathBuf, sync::Arc};
type Environment = usize;
// Frozen derived comparator from 9e945b04d; never calls Value::eq.
#[derive(Debug, PartialEq, Eq)]
enum Legacy {
    Prefix(String, bool, Option<u64>),
    Promise(Box<Legacy>),
    Evaluated(Box<Legacy>, bool),
    Aggregate(Vec<Legacy>),
    // Reachable references retained for effects, not candidate runtime values.
    References(Vec<Legacy>),
    // Direct object binds are scalars; property reads may expose these values.
    Object(Vec<Legacy>),
    Record(Vec<Legacy>, Vec<String>),
    // Alternative binding values; unlike Aggregate, this is not a container.
    Joined(Vec<Legacy>),
    // Candidate runtime values with an implicit unknown alternative.
    Possible(Vec<Legacy>),
    Arguments(u64),
    Array(u64),
    Function(Arc<Function>, PathBuf, Environment),
    Number(i64),
    Boolean(bool),
    Failure(&'static str),
    Unknown,
    Primitive,
    Unsupported,
    SlotDeletion,
}
impl From<&Value> for Legacy {
    fn from(value: &Value) -> Self {
        match value {
            Value::Prefix(a, b, c) => Self::Prefix(a.clone(), *b, *c),
            Value::Promise(a) => Self::Promise(Box::new(Self::from(a.as_ref()))),
            Value::Evaluated(a, b) => Self::Evaluated(Box::new(Self::from(a.as_ref())), *b),
            Value::Aggregate(a) => Self::Aggregate(a.iter().map(Self::from).collect()),
            Value::References(a) => Self::References(a.iter().map(Self::from).collect()),
            Value::Object(a) => Self::Object(a.iter().map(Self::from).collect()),
            Value::Record(a, b) => Self::Record(a.iter().map(Self::from).collect(), b.clone()),
            Value::Joined(a) => Self::Joined(a.iter().map(Self::from).collect()),
            Value::Possible(a) => Self::Possible(a.iter().map(Self::from).collect()),
            Value::Arguments(a) => Self::Arguments(*a),
            Value::Array(a) => Self::Array(*a),
            Value::Function(a, b, c) => Self::Function(a.clone(), b.as_ref().to_path_buf(), *c),
            Value::Number(a) => Self::Number(*a),
            Value::Boolean(a) => Self::Boolean(*a),
            Value::Failure(a) => Self::Failure(a),
            Value::Unknown => Self::Unknown,
            Value::Primitive => Self::Primitive,
            Value::Unsupported => Self::Unsupported,
            Value::SlotDeletion => Self::SlotDeletion,
        }
    }
}
pub(super) fn equal(a: &Value, b: &Value) -> bool {
    Legacy::from(a) == Legacy::from(b)
}
pub(super) fn describe(value: &Value) -> String {
    format!("{:?}", Legacy::from(value))
}
