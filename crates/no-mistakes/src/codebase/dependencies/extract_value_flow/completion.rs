use super::*;

pub(super) fn sequence_completes(statements: &[Statement<'_>]) -> bool {
    sequence_completion(statements).0
}

pub(super) fn sequence_completion(statements: &[Statement<'_>]) -> (bool, Option<u32>) {
    for (index, statement) in statements.iter().enumerate() {
        let (completes, nested_cutoff) = statement_completion(statement);
        if !completes {
            let cutoff = nested_cutoff
                .or_else(|| (index + 1 < statements.len()).then_some(statement.span().end));
            return (false, cutoff);
        }
    }
    (true, None)
}

fn statement_completion(statement: &Statement<'_>) -> (bool, Option<u32>) {
    match statement {
        // Propagate only a sequential block's cutoff. Branch-local cutoffs
        // stay confined to their branch because another path may reach it.
        Statement::BlockStatement(block) => sequence_completion(&block.body),
        _ => (statement_completes(statement), None),
    }
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

impl ImportCollector {
    pub(in super::super) fn record_value_fallthrough(
        &mut self,
        statements: &[Statement<'_>],
    ) -> Option<u32> {
        if !self.collect_value_flow {
            return self.unreachable_value_binding_after;
        }
        let previous_cutoff = self.unreachable_value_binding_after;
        let (completes, cutoff) = sequence_completion(statements);
        self.unreachable_value_binding_after = cutoff;
        if completes {
            if let Some(owner) = self.current_function_id() {
                self.value_flow
                    .returns
                    .push((owner, ValueExpression::Unknown));
            }
        }
        previous_cutoff
    }
}
