fn visit_statement_with_type_binding<'a>(
    collector: &mut ImportCollector,
    statement: &Statement<'a>,
) {
    #[cfg(all(test, feature = "test-instrumentation"))]
    if collector.statement_mode.original() {
        statement_legacy::visit_statement_with_type_binding(collector, statement);
        return;
    }
    let previous = if collector.collect_value_flow {
        Some(record_replacement_statement(collector, statement))
    } else {
        None
    };
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

fn visit_expression_with_annex_b_initializer<'a>(
    collector: &mut ImportCollector,
    expression: &Expression<'a>,
) {
    if let Some((_, outer_conditional, outer_receiver_conditional)) = collector
        .annex_b_for_in_initializer
        .filter(|(span, _, _)| !collector.suppress_imports && *span == expression.span())
    {
        let initializer = collector.annex_b_for_in_initializer.take();
        let previous = std::mem::replace(&mut collector.conditional_execution, outer_conditional);
        let previous_receiver = std::mem::replace(
            &mut collector.receiver_write_is_conditional,
            outer_receiver_conditional,
        );
        walk::walk_expression(collector, expression);
        collector.receiver_write_is_conditional = previous_receiver;
        collector.conditional_execution = previous;
        collector.annex_b_for_in_initializer = initializer;
    } else {
        walk::walk_expression(collector, expression);
    }
}

include!("extract_replacement_statement.rs");
