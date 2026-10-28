use oxc_ast::ast::Statement;

/// Only a branch whose sole statement throws is unreachable at the join.
/// The walk still visits its expression; try/catch invalidates the joined facts.
pub(super) fn throws(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => {
            matches!(block.body.as_slice(), [Statement::ThrowStatement(_)])
        }
        _ => false,
    }
}
