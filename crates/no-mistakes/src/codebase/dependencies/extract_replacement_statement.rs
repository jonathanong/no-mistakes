// Called only after the enclosing visitor admits value-flow demand.
fn record_replacement_statement(
    collector: &mut ImportCollector,
    statement: &Statement<'_>,
) -> Option<(u32, u32)> {
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
}
