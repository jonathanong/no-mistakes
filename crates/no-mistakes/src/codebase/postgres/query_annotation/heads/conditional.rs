//! Finite original-AST heads; unknown or transaction-boundary heads abstain.
use super::syntax::{bounded_head, Head};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ConditionalExpression, Expression};

// The former depth-eight limit could visit 255 conditional nodes. Keep that
// work bound while admitting long finite chains without recursive descent.
const MAX_CONDITIONAL_NODES: usize = 255;

pub(super) fn head<'a>(
    branch: &'a ConditionalExpression<'a>,
    depth: u8,
    transformed: bool,
) -> Head {
    if depth == 0 || transformed {
        return Head::Unproven;
    }
    let left = unwrap_ts_wrappers(&branch.consequent);
    let right = unwrap_ts_wrappers(&branch.alternate);
    if !matches!(left, Expression::ConditionalExpression(_))
        && !matches!(right, Expression::ConditionalExpression(_))
    {
        return join(
            bounded_head(left, depth - 1),
            bounded_head(right, depth - 1),
        );
    }
    let mut remaining = MAX_CONDITIONAL_NODES - 1;
    let mut pending = vec![right, left];
    let mut result = Head::Literal;
    while let Some(expression) = pending.pop() {
        let expression = unwrap_ts_wrappers(expression);
        if let Expression::ConditionalExpression(branch) = expression {
            if remaining == 0 {
                return Head::Unproven;
            }
            remaining -= 1;
            pending.push(&branch.alternate);
            pending.push(&branch.consequent);
        } else {
            // Leaf classification retains transformation and tag lifetime rules.
            // A conditional hidden under concatenation/append still abstains.
            result = join(result, bounded_head(expression, depth - 1));
            if matches!(result, Head::Unproven) {
                return result;
            }
        }
    }
    result
}

fn join(left: Head, right: Head) -> Head {
    match (left, right) {
        (Head::Literal, Head::Literal) => Head::Literal,
        (Head::Literal | Head::Tag, Head::Literal | Head::Tag) => Head::Tag,
        (
            Head::Literal | Head::Tag | Head::AppendTag,
            Head::Literal | Head::Tag | Head::AppendTag,
        ) => Head::AppendTag,
        _ => Head::Unproven,
    }
}
