#[derive(PartialEq)]
enum SwitchLiteral<'a> {
    Boolean(bool),
    Number(f64),
    String(std::borrow::Cow<'a, str>, bool),
    Null,
}

fn switch_literal<'b>(expression: &'b Expression<'_>) -> Option<SwitchLiteral<'b>> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::BooleanLiteral(value) => Some(SwitchLiteral::Boolean(value.value)),
        Expression::NumericLiteral(value) => Some(SwitchLiteral::Number(value.value)),
        Expression::StringLiteral(value) => Some(SwitchLiteral::String(
            crate::ast::js_literal(&value),
            crate::ast::js_literal_has_lone_surrogate(value),
        )),
        Expression::NullLiteral(_) => Some(SwitchLiteral::Null),
        _ => None,
    }
}

fn record_dead_switch_cases(
    collector: &mut ImportCollector,
    switch: &oxc_ast::ast::SwitchStatement<'_>,
) {
    if !collector.collect_call_eligibility {
        return;
    }
    let Some(discriminant) = switch_literal(&switch.discriminant) else {
        return;
    };
    let mut selected = None;
    let mut default = None;
    for (index, case) in switch.cases.iter().enumerate() {
        if let Some(test) = &case.test {
            let Some(value) = switch_literal(test) else {
                return;
            };
            if value == discriminant && selected.is_none() {
                selected = Some(index);
            }
        } else {
            default = Some(index);
        }
    }
    // A default before a matching case is still skipped. Unknown identities
    // above retain the existing syntactic treatment instead of guessing.
    let start = selected.or(default);
    let mut stopped = false;
    for (index, case) in switch.cases.iter().enumerate() {
        if start.is_none_or(|start| index < start) || stopped {
            for statement in &case.consequent {
                record_dead_call_span(collector, statement.span());
            }
        } else {
            stopped = record_switch_statement_suffix(collector, &case.consequent);
        }
    }
}

fn record_switch_statement_suffix(
    collector: &mut ImportCollector,
    statements: &[Statement<'_>],
) -> bool {
    let mut exited = false;
    for statement in statements {
        if exited {
            record_dead_call_span(collector, statement.span());
            continue;
        }
        exited = match statement {
            Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_) => true,
            Statement::BlockStatement(block) => {
                record_switch_statement_suffix(collector, &block.body)
            }
            // A nested loop's break or callable's return belongs to that owner.
            _ => false,
        };
    }
    exited
}
