use super::*;

// Match only immutable root shapes; recursive property/depth proofs stay uncached.
pub(super) fn same_member_root(left: &ValueExpression, right: &ValueExpression) -> bool {
    match (left, right) {
        (
            ValueExpression::Named {
                name: left,
                scope: left_scope,
                ..
            },
            ValueExpression::Named {
                name: right,
                scope: right_scope,
                ..
            },
        ) => left == right && left_scope == right_scope,
        (
            ValueExpression::Member(left, left_member),
            ValueExpression::Member(right, right_member),
        ) => {
            left_member == right_member
                && matches!((left.as_ref(), right.as_ref()),
                (ValueExpression::Named { name: left, scope: left_scope, .. },
                 ValueExpression::Named { name: right, scope: right_scope, .. })
                    if left == right && left_scope == right_scope)
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "replacement_member_key/tests.rs"]
mod tests;
