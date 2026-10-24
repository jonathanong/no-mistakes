use super::{Value, Values};
mod direct;
#[cfg(not(test))]
mod hooks;
#[cfg(test)]
#[path = "equality/test_hooks.rs"]
mod hooks;
mod pair_memo;
use direct::atomic;
mod prefix_order;

#[derive(Default)]
struct Comparison {
    completed: pair_memo::Memo,
}

// Compare a cheap prefix once. The first unresolved nested pair owns one memo
// for every remaining sibling, including descendants of distinct unique parents.
pub(super) fn values(a: &Values, b: &Values) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut pairs = a.iter().zip(b.iter());
    while let Some((left, right)) = pairs.next() {
        if let Some(equal) = atomic(left, right) {
            hooks::fallback!();
            if !equal {
                return false;
            }
        } else if let Some(equal) = fast(left, right) {
            if !equal {
                return false;
            }
        } else {
            let mut context = Comparison::default();
            return context.value(left, right) && pairs.all(|(a, b)| context.value(a, b));
        }
    }
    true
}

// This small admission helper must not add an out-of-line call to scalar paths.
#[inline(always)]
fn fast(a: &Value, b: &Value) -> Option<bool> {
    let equal = if let Some(equal) = atomic(a, b) {
        equal
    } else {
        match (a, b) {
            (Value::Promise(a), Value::Promise(b)) => fast(a, b)?,
            (Value::Evaluated(a, done), Value::Evaluated(b, other_done)) => {
                fast(a, b)? && done == other_done
            }
            (Value::Record(a, labels), Value::Record(b, other_labels)) => {
                if a.identity() != b.identity() {
                    return None;
                }
                labels == other_labels
            }
            (Value::Aggregate(a), Value::Aggregate(b))
            | (Value::References(a), Value::References(b))
            | (Value::Object(a), Value::Object(b))
            | (Value::Joined(a), Value::Joined(b))
            | (Value::Possible(a), Value::Possible(b)) => {
                if a.identity() != b.identity() {
                    return None;
                }
                true
            }
            _ => false,
        }
    };
    hooks::fallback!();
    Some(equal)
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

#[cfg(test)]
mod tests;
