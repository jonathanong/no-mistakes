use super::{hooks, Value};
// Shared Scope snapshots can return the very same Value reference. Avoid descending
// into strings and record labels; distinct references retain structural equality.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if hooks::identity!(self, other) {
            return true;
        }
        hooks::fallback!();
        // Keep this exhaustive so new variants must explicitly choose their comparator.
        match self {
            Self::Prefix(..) | Self::Evaluated(..) | Self::Record(..) | Self::Function(..) => {
                composite(self, other)
            }
            Self::Promise(_)
            | Self::Aggregate(_)
            | Self::References(_)
            | Self::Object(_)
            | Self::Joined(_)
            | Self::Possible(_) => nested(self, other),
            Self::Arguments(_)
            | Self::Array(_)
            | Self::Number(_)
            | Self::Boolean(_)
            | Self::Failure(_) => scalar(self, other),
            Self::Unknown => matches!(other, Self::Unknown),
            Self::Primitive => matches!(other, Self::Primitive),
            Self::Unsupported => matches!(other, Self::Unsupported),
            Self::SlotDeletion => matches!(other, Self::SlotDeletion),
        }
    }
}
fn composite(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Prefix(a, b, c), Value::Prefix(a2, b2, c2)) => {
            hooks::prefix!(a, b, c, a2, b2, c2)
        }
        (Value::Evaluated(a, b), Value::Evaluated(a2, b2)) => a == a2 && b == b2,
        (Value::Record(a, b), Value::Record(a2, b2)) => a == a2 && b == b2,
        (Value::Function(a, b, c), Value::Function(a2, b2, c2)) => a == a2 && b == b2 && c == c2,
        _ => false,
    }
}
fn nested(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Promise(a), Value::Promise(b)) => a == b,
        (Value::Aggregate(a), Value::Aggregate(b))
        | (Value::References(a), Value::References(b))
        | (Value::Object(a), Value::Object(b))
        | (Value::Joined(a), Value::Joined(b))
        | (Value::Possible(a), Value::Possible(b)) => a == b,
        _ => false,
    }
}
fn scalar(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Arguments(a), Value::Arguments(b)) | (Value::Array(a), Value::Array(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Failure(a), Value::Failure(b)) => a == b,
        _ => false,
    }
}
#[inline]
pub(super) fn atomic(a: &Value, b: &Value) -> Option<bool> {
    Some(match a {
        Value::Prefix(..) | Value::Function(..) => composite(a, b),
        Value::Arguments(..)
        | Value::Array(..)
        | Value::Number(..)
        | Value::Boolean(..)
        | Value::Failure(..) => scalar(a, b),
        Value::Unknown => matches!(b, Value::Unknown),
        Value::Primitive => matches!(b, Value::Primitive),
        Value::Unsupported => matches!(b, Value::Unsupported),
        Value::SlotDeletion => matches!(b, Value::SlotDeletion),
        Value::Evaluated(..)
        | Value::Record(..)
        | Value::Promise(..)
        | Value::Aggregate(..)
        | Value::References(..)
        | Value::Object(..)
        | Value::Joined(..)
        | Value::Possible(..) => return None,
    })
}
