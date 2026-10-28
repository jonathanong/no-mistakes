//! Finite original-AST heads; unknown or transaction-boundary heads abstain.
use super::syntax::{bounded_head, Head};
use oxc_ast::ast::ConditionalExpression;

pub(super) fn head<'a>(
    branch: &'a ConditionalExpression<'a>,
    depth: u8,
    transformed: bool,
) -> Head {
    if depth == 0 || transformed {
        return Head::Unproven;
    }
    let left = bounded_head(&branch.consequent, depth - 1);
    let right = bounded_head(&branch.alternate, depth - 1);
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
