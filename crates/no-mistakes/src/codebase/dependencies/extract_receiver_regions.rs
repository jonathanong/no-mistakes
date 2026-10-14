// This is the normal block walk, with a transient token for consecutive plain
// assignment statements. No child statement can lend certainty to its parent.
fn visit_guarded_receiver_arm<'a>(collector: &mut ImportCollector, statement: &Statement<'a>, guarded: bool) {
    let Statement::BlockStatement(block) = statement else {
        collector.visit_statement(statement);
        return;
    };
    if !guarded || !collector.collect_value_flow { collector.visit_statement(statement); return; }
    collector.receiver_conditional_region = block.span.start.checked_add(1)
        .and_then(std::num::NonZeroU32::new).map(|token| (token, u32::MAX));
    collector.visit_statement(statement);
    collector.receiver_conditional_region = None;
}

fn visit_statements_with_receiver_region<'a>(collector: &mut ImportCollector, statements: &oxc_allocator::Vec<'a, Statement<'a>>) {
    if !collector.receiver_conditional_region.is_some_and(|(_, admitted)| admitted == u32::MAX) {
        walk::walk_statements(collector, statements);
        return;
    }
    let mut token = None;
    for statement in statements {
        let start = match statement {
            Statement::ExpressionStatement(statement) => match &statement.expression {
                Expression::AssignmentExpression(assignment)
                    if assignment.operator == oxc_syntax::operator::AssignmentOperator::Assign => Some(assignment.span.start),
                _ => None,
            },
            _ => None,
        };
        if let Some(start) = start {
            token = token.or_else(|| start.checked_add(1).and_then(std::num::NonZeroU32::new));
            collector.receiver_conditional_region = token.map(|token| (token, start));
        } else {
            token = None;
            collector.receiver_conditional_region = None;
        }
        collector.visit_statement(statement);
        if collector.receiver_conditional_region.is_none() { token = None; }
        collector.receiver_conditional_region = None;
    }
}
