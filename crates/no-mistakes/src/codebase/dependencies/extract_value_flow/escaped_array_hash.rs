use super::*;
use std::hash::{Hash, Hasher};

pub(super) fn fingerprint((slot, value): &(usize, ValueExpression)) -> u64 {
    let mut hash = rustc_hash::FxHasher::default();
    slot.hash(&mut hash);
    expression(value, &mut hash);
    hash.finish()
}
fn expression(value: &ValueExpression, hash: &mut impl Hasher) {
    std::mem::discriminant(value).hash(hash);
    match value {
        ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } => {
            (name, scope, offset, context).hash(hash);
        }
        ValueExpression::Call(start, end) => (start, end).hash(hash),
        ValueExpression::Callable(id) | ValueExpression::Super(id) => id.hash(hash),
        ValueExpression::Awaited(value) => expression(value, hash),
        ValueExpression::Member(value, member) => {
            expression(value, hash);
            member.hash(hash);
        }
        ValueExpression::Object(fields, null_prototype) => {
            null_prototype.hash(hash);
            properties(fields, hash);
        }
        ValueExpression::Array(fields) => properties(fields, hash),
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            expression(left, hash);
            expression(right, hash);
        }
        ValueExpression::Undefined | ValueExpression::Null | ValueExpression::Unknown => {}
    }
}
fn properties(fields: &FxHashMap<String, ValueExpression>, hash: &mut impl Hasher) {
    // Map equality ignores insertion order, so hashing must do the same.
    let mut keys = fields.keys().collect::<Vec<_>>();
    keys.sort_unstable();
    keys.len().hash(hash);
    for key in keys {
        key.hash(hash);
        expression(&fields[key], hash);
    }
}

#[cfg(test)]
#[path = "escaped_array_hash_tests.rs"]
mod tests;
