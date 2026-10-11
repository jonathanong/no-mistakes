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
    collector.visit_expression(&switch.discriminant);
    let pushed = collector.push_lexical_scope();
    for case in &switch.cases {
        predeclare_function_declarations(collector, &case.consequent);
    }
    with_conditional_execution(collector, true, |collector| {
        collector.visit_switch_cases(&switch.cases);
    });
    collector.pop_lexical_scope(pushed);
}

fn visit_for_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForStatement<'a>,
) {
    collector.record_per_item_range(statement.body.span(), collector.current_function_id(), "for");
    let pushed = collector.push_lexical_scope();
    if let Some(init) = &statement.init {
        collector.visit_for_statement_init(init);
    }
    if let Some(test) = &statement.test {
        collector.visit_expression(test);
    }
    if let Some(update) = &statement.update {
        with_conditional_execution(collector, true, |collector| collector.visit_expression(update));
    }
    let conditional = statement.test.as_ref().is_some_and(|test| literal_truthiness(test) != Some(true));
    with_conditional_execution(collector, conditional, |collector| collector.visit_statement(&statement.body));
    collector.pop_lexical_scope(pushed);
}

fn visit_for_in_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForInStatement<'a>,
) {
    collector.record_per_item_range(statement.body.span(), collector.current_function_id(), "for-in");
    let pushed = collector.push_lexical_scope();
    invalidate_for_statement_assignment_target(collector, &statement.left);
    // The binding target (including destructuring defaults) runs per iteration.
    with_conditional_execution(collector, true, |collector| collector.visit_for_statement_left(&statement.left));
    collector.visit_expression(&statement.right);
    with_conditional_execution(collector, true, |collector| collector.visit_statement(&statement.body));
    collector.pop_lexical_scope(pushed);
}

fn visit_for_of_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForOfStatement<'a>,
) {
    collector.record_per_item_range(statement.body.span(), collector.current_function_id(), "for-of");
    let pushed = collector.push_lexical_scope();
    invalidate_for_statement_assignment_target(collector, &statement.left);
    with_conditional_execution(collector, true, |collector| collector.visit_for_statement_left(&statement.left));
    collector.visit_expression(&statement.right);
    with_conditional_execution(collector, true, |collector| collector.visit_statement(&statement.body));
    collector.pop_lexical_scope(pushed);
}

fn invalidate_for_statement_assignment_target(
    collector: &mut ImportCollector,
    left: &ForStatementLeft<'_>,
) {
    let Some(target) = left.as_assignment_target() else {
        return;
    };
    for name in assignment_target_names(target) {
        collector.record_reassigned_callable_alias(&name, target.span().start);
    }
}

fn visit_catch_clause_with_scope<'a>(collector: &mut ImportCollector, clause: &CatchClause<'a>) {
    let pushed = collector.push_lexical_scope();
    if let Some(param) = &clause.param {
        collector.add_binding_names(&param.pattern);
    }
    with_conditional_execution(collector, true, |collector| walk::walk_catch_clause(collector, clause));
    collector.pop_lexical_scope(pushed);
}

fn visit_while_statement_per_item<'a>(collector: &mut ImportCollector, statement: &oxc_ast::ast::WhileStatement<'a>) {
    collector.record_per_item_range(statement.body.span(), collector.current_function_id(), "while");
    collector.visit_expression(&statement.test);
    with_conditional_execution(collector, literal_truthiness(&statement.test) != Some(true), |collector| {
        collector.visit_statement(&statement.body);
    });
}
fn visit_do_while_statement_per_item<'a>(collector: &mut ImportCollector, statement: &oxc_ast::ast::DoWhileStatement<'a>) {
    collector.record_per_item_range(statement.body.span(), collector.current_function_id(), "do");
    collector.visit_statement(&statement.body);
    // Only prove simple bodies: explicit exits can skip even the first test.
    with_conditional_execution(collector, !do_body_reaches_test(&statement.body), |collector| collector.visit_expression(&statement.test));
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
