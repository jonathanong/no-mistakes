// Additional eligibility facts never remove the historical syntactic call edges.
fn record_dead_statement_suffix(collector: &mut ImportCollector, statements: &[Statement<'_>]) {
    if !collector.collect_call_eligibility {
        return;
    }
    if let Some(index) = statements.iter().position(|statement| {
        matches!(
            statement,
            Statement::ReturnStatement(_)
                | Statement::ThrowStatement(_)
                | Statement::BreakStatement(_)
                | Statement::ContinueStatement(_)
        )
    }) {
        if let (Some(first), Some(last)) = (statements.get(index + 1), statements.last()) {
            collector.unreachable_call_ranges.push((
                collector.current_function_id(),
                first.span().start,
                last.span().end,
            ));
        }
    }
}
fn record_dead_branch(
    collector: &mut ImportCollector,
    truthy: Option<bool>,
    consequent: &Statement<'_>,
    alternate: Option<&Statement<'_>>,
) {
    if !collector.collect_call_eligibility {
        return;
    }
    let dead = match truthy {
        Some(false) => Some(consequent),
        Some(true) => alternate,
        None => None,
    };
    if let Some(dead) = dead {
        collector.unreachable_call_ranges.push((
            collector.current_function_id(),
            dead.span().start,
            dead.span().end,
        ));
    }
}

fn record_dead_call_span(collector: &mut ImportCollector, span: oxc_span::Span) {
    collector
        .unreachable_call_ranges
        .push((collector.current_function_id(), span.start, span.end));
}
fn record_dead_expression_arm(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::ConditionalExpression<'_>,
) {
    if !collector.collect_call_eligibility {
        return;
    }
    match literal_truthiness(&expression.test) {
        Some(true) => record_dead_call_span(collector, expression.alternate.span()),
        Some(false) => record_dead_call_span(collector, expression.consequent.span()),
        None => {}
    }
}
fn record_dead_logical_arm(
    collector: &mut ImportCollector,
    expression: &oxc_ast::ast::LogicalExpression<'_>,
) {
    if !collector.collect_call_eligibility {
        return;
    }
    let dead = match expression.operator {
        oxc_ast::ast::LogicalOperator::And => literal_truthiness(&expression.left) == Some(false),
        oxc_ast::ast::LogicalOperator::Or => literal_truthiness(&expression.left) == Some(true),
        oxc_ast::ast::LogicalOperator::Coalesce => matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&expression.left),
            Expression::BooleanLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::BigIntLiteral(_)
        ),
    };
    if dead {
        record_dead_call_span(collector, expression.right.span());
    }
}
fn record_dead_for_body(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForStatement<'_>,
) {
    if !collector.collect_call_eligibility {
        return;
    }
    if statement.test.as_ref().and_then(literal_truthiness) == Some(false) {
        record_dead_call_span(collector, statement.body.span());
        if let Some(update) = &statement.update {
            record_dead_call_span(collector, update.span());
        }
    }
}

// Keep execution exclusions in final fact ownership form, indexed by owner and
// start byte. Lookup must not scan every dead branch for every call occurrence.
fn normalize_unreachable_call_ranges(
    mut ranges: Vec<UnreachableCallRange>,
) -> Vec<UnreachableCallRange> {
    if ranges.len() < 2 {
        return ranges;
    }
    ranges.sort_unstable();
    let mut written = 0usize;
    for read in 0..ranges.len() {
        let next = ranges[read];
        if written > 0 && ranges[written - 1].0 == next.0 && ranges[written - 1].2 >= next.1 {
            ranges[written - 1].2 = ranges[written - 1].2.max(next.2);
        } else {
            ranges[written] = next;
            written += 1;
        }
    }
    ranges.truncate(written);
    ranges
}
