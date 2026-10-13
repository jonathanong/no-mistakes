use super::*;

pub(super) fn sequence_completes(statements: &[Statement<'_>]) -> bool {
    statements.iter().all(statement_completes)
}
fn statement_completes(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => false,
        Statement::BlockStatement(block) => sequence_completes(&block.body),
        Statement::IfStatement(branch) => {
            statement_completes(&branch.consequent)
                || branch
                    .alternate
                    .as_ref()
                    .is_none_or(|alternate| statement_completes(alternate))
        }
        _ => true,
    }
}
