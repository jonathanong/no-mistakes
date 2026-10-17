use super::*;

pub(super) fn visit_statement_with_type_binding<'a>(
    collector: &mut ImportCollector,
    statement: &Statement<'a>,
) {
    let previous = collector.collect_value_flow.then(|| {
        #[cfg(feature = "test-instrumentation")]
        {
            collector.value_flow.replacement_statement_checks += 1;
        }
        if matches!(
            statement,
            Statement::ThrowStatement(_)
                | Statement::BreakStatement(_)
                | Statement::ContinueStatement(_)
        ) || (collector.current_function_id().is_some()
            || collector.replacement_region.is_some())
            && !matches!(
                statement,
                Statement::BlockStatement(_)
                    | Statement::ExpressionStatement(_)
                    | Statement::VariableDeclaration(_)
                    | Statement::FunctionDeclaration(_)
                    | Statement::EmptyStatement(_)
                    | Statement::DebuggerStatement(_)
            )
        {
            collector.invalidate_replacement_execution();
        }
        let direct = match statement {
            Statement::ExpressionStatement(statement) => {
                match crate::codebase::ts_source::unwrap_ts_wrappers(&statement.expression) {
                    Expression::CallExpression(call) => Some((call.span.start, call.span.end)),
                    _ => None,
                }
            }
            _ => None,
        };
        std::mem::replace(&mut collector.replacement_statement_call, direct)
    });
    record_statement_type_binding(collector, statement);
    let previous_cutoff = collector.unreachable_value_binding_after;
    walk::walk_statement(collector, statement);
    if let Some(previous) = previous {
        collector.replacement_statement_call = previous;
    }
    // A terminal only cuts off its own sequential siblings, not another branch.
    collector.unreachable_value_binding_after = previous_cutoff;
    if collector.collect_value_flow
        && matches!(
            statement,
            Statement::ReturnStatement(_) | Statement::ThrowStatement(_)
        )
    {
        collector.unreachable_value_binding_after =
            Some(previous_cutoff.map_or(statement.span().end, |cutoff| {
                cutoff.min(statement.span().end)
            }));
    }
}
