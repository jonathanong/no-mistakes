use super::*;

pub(super) fn sequence_completes(statements: &[Statement<'_>]) -> bool {
    sequence_completion(statements).0
}

pub(super) fn sequence_completion(statements: &[Statement<'_>]) -> (bool, Option<u32>) {
    for (index, statement) in statements.iter().enumerate() {
        if !statement_completes(statement) {
            let cutoff = (index + 1 < statements.len()).then_some(statement.span().end);
            return (false, cutoff);
        }
    }
    (true, None)
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
        Statement::SwitchStatement(switch) => {
            if !switch.cases.iter().any(|case| case.test.is_none()) {
                return true;
            }
            let mut suffix = true;
            let mut any = false;
            for case in switch.cases.iter().rev() {
                suffix = has_switch_break(&case.consequent)
                    || (sequence_completes(&case.consequent) && suffix);
                any |= suffix;
            }
            any
        }
        _ => true,
    }
}

fn has_switch_break(statements: &[Statement<'_>]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::BreakStatement(_) => true,
        Statement::BlockStatement(block) => has_switch_break(&block.body),
        Statement::IfStatement(branch) => {
            has_switch_break(std::slice::from_ref(&branch.consequent))
                || branch
                    .alternate
                    .as_ref()
                    .is_some_and(|alternate| has_switch_break(std::slice::from_ref(alternate)))
        }
        _ => false,
    })
}
