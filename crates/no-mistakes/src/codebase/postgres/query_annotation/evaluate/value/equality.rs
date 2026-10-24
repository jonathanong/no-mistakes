use super::{Value, Values};
#[cfg(not(test))]
mod hooks;
#[cfg(test)]
#[path = "equality/test_hooks.rs"]
mod hooks;
mod pair_memo;
mod prefix_order;

#[derive(Default)]
struct Comparison {
    completed: pair_memo::Memo,
}

pub(super) fn values(a: &Values, b: &Values) -> bool {
    Comparison::default().values(a, b)
}

// A comparison owns its memo: immutable container addresses remain live through
// this call, while mutable argument objects are compared only by their IDs.
impl Comparison {
    fn values(&mut self, a: &Values, b: &Values) -> bool {
        if a.identity() == b.identity() {
            return true;
        }
        if a.len() != b.len() {
            return false;
        }
        // Unique containers cannot recur; shallow comparisons keep pairs inline.
        let repeated = hooks::memoize() && a.is_shared() && b.is_shared();
        let pair = (a.identity() as usize, b.identity() as usize);
        if repeated && self.completed.contains(pair) {
            return true;
        }
        let equal = a.iter().zip(b.iter()).all(|(a, b)| self.value(a, b));
        if repeated && equal {
            self.completed.insert(pair);
        }
        equal
    }

    fn value(&mut self, a: &Value, b: &Value) -> bool {
        // Shared containers returned before iteration; these slots are distinct.
        hooks::fallback!();
        if let Some(equal) = atomic(a, b) {
            return equal;
        }
        self.nested(a, b)
    }

    fn nested(&mut self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Evaluated(a, b), Value::Evaluated(a2, b2)) => self.value(a, a2) && b == b2,
            (Value::Record(a, b), Value::Record(a2, b2)) => self.values(a, a2) && b == b2,
            (Value::Promise(a), Value::Promise(b)) => self.value(a, b),
            (Value::Aggregate(a), Value::Aggregate(b))
            | (Value::References(a), Value::References(b))
            | (Value::Object(a), Value::Object(b))
            | (Value::Joined(a), Value::Joined(b))
            | (Value::Possible(a), Value::Possible(b)) => self.values(a, b),
            _ => false,
        }
    }
}

// Shared Scope snapshots still avoid string/record work before initializing a memo.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if hooks::identity!(self, other) {
            return true;
        }
        hooks::fallback!();
        if let Some(equal) = atomic(self, other) {
            return equal;
        }
        Comparison::default().nested(self, other)
    }
}

fn atomic(a: &Value, b: &Value) -> Option<bool> {
    Some(match a {
        Value::Prefix(text, complete, id) => match b {
            Value::Prefix(other, done, other_id) => {
                hooks::prefix!(text, complete, id, other, done, other_id)
            }
            _ => false,
        },
        Value::Function(function, path, env) => {
            matches!(b, Value::Function(other, other_path, other_env) if function == other && path == other_path && env == other_env)
        }
        Value::Arguments(a) => matches!(b, Value::Arguments(other) if a == other),
        Value::Array(a) => matches!(b, Value::Array(other) if a == other),
        Value::Number(a) => matches!(b, Value::Number(other) if a == other),
        Value::Boolean(a) => matches!(b, Value::Boolean(other) if a == other),
        Value::Failure(a) => matches!(b, Value::Failure(other) if a == other),
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
#[cfg(test)]
mod tests;
