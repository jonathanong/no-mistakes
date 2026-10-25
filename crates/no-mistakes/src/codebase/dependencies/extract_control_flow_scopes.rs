include!("extract_expression_statement_walk.rs");

fn visit_block_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    block: &BlockStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    predeclare_function_declarations(collector, &block.body);
    walk::walk_block_statement(collector, block);
    collector.pop_lexical_scope(pushed);
}

fn visit_switch_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    switch: &oxc_ast::ast::SwitchStatement<'a>,
) {
    // Case consequents share one lexical environment, while the discriminant
    // is evaluated outside it.
    record_dead_switch_cases(collector, switch);
    collector.visit_expression(&switch.discriminant);
    let pushed = collector.push_lexical_scope();
    for case in &switch.cases {
        predeclare_function_declarations(collector, &case.consequent);
    }
    let mut previous_test_may_match = false;
    for case in &switch.cases {
        // A leading default does not run its body before case tests. The first
        // actual test always runs; later tests may be skipped after a match.
        if let Some(test) = &case.test {
            with_conditional_execution(collector, previous_test_may_match, |collector| {
                collector.visit_expression(test);
            });
            previous_test_may_match = true;
        }
    }
    for case in &switch.cases {
        let previous_cutoff = collector.unreachable_value_binding_after;
        with_conditional_execution(collector, true, |collector| {
            for statement in &case.consequent {
                collector.visit_statement(statement);
            }
        });
        collector.unreachable_value_binding_after = previous_cutoff;
    }
    collector.pop_lexical_scope(pushed);
}

fn visit_catch_clause_with_scope<'a>(collector: &mut ImportCollector, clause: &CatchClause<'a>) {
    let pushed = collector.push_lexical_scope();
    if let Some(param) = &clause.param {
        collector.add_binding_names(&param.pattern);
    }
    with_conditional_execution(collector, true, |collector| {
        walk::walk_catch_clause(collector, clause)
    });
    collector.pop_lexical_scope(pushed);
}

fn visit_while_statement_per_item<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::WhileStatement<'a>,
) {
    collector.record_per_item_range(
        statement.body.span(),
        collector.current_function_id(),
        "while",
    );
    collector.visit_expression(&statement.test);
    with_conditional_execution(
        collector,
        literal_truthiness(&statement.test) != Some(true),
        |collector| {
            with_conditional_receiver_write(collector, |collector| {
                with_replacement_region(
                    collector,
                    statement.body.span(),
                    false,
                    literal_truthiness(&statement.test) != Some(false),
                    |collector| collector.visit_statement(&statement.body),
                );
            });
        },
    );
}
fn visit_do_while_statement_per_item<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::DoWhileStatement<'a>,
) {
    collector.record_per_item_range(statement.body.span(), collector.current_function_id(), "do");
    with_conditional_receiver_write(collector, |collector| {
        with_replacement_region(
            collector,
            statement.body.span(),
            false,
            literal_truthiness(&statement.test) != Some(false),
            |collector| collector.visit_statement(&statement.body),
        );
    });
    // Only prove simple bodies: explicit exits can skip even the first test.
    with_conditional_execution(
        collector,
        !do_body_reaches_test(&statement.body),
        |collector| collector.visit_expression(&statement.test),
    );
}

fn do_body_reaches_test(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::EmptyStatement(_)
        | Statement::ExpressionStatement(_)
        | Statement::VariableDeclaration(_)
        | Statement::FunctionDeclaration(_)
        | Statement::DebuggerStatement(_) => true,
        Statement::BlockStatement(block) => block.body.iter().all(do_body_reaches_test),
        Statement::IfStatement(branch) => {
            do_body_reaches_test(&branch.consequent)
                && branch.alternate.as_ref().is_none_or(do_body_reaches_test)
        }
        // An unlabeled continue in this body proceeds to the do-while test.
        Statement::ContinueStatement(statement) => statement.label.is_none(),
        // Nested loops, labels, switches and exception handling need a fuller
        // completion model. Retain their conservative classification.
        _ => false,
    }
}
