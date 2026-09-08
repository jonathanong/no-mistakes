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
    collector.visit_switch_cases(&switch.cases);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    if let Some(init) = &statement.init {
        collector.visit_for_statement_init(init);
    }
    if let Some(test) = &statement.test {
        collector.visit_expression(test);
    }
    if let Some(update) = &statement.update {
        collector.visit_expression(update);
    }
    collector.visit_statement(&statement.body);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_in_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForInStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    invalidate_for_statement_assignment_target(collector, &statement.left);
    collector.visit_for_statement_left(&statement.left);
    collector.visit_expression(&statement.right);
    collector.visit_statement(&statement.body);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_of_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForOfStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    invalidate_for_statement_assignment_target(collector, &statement.left);
    collector.visit_for_statement_left(&statement.left);
    collector.visit_expression(&statement.right);
    collector.visit_statement(&statement.body);
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
        collector.record_reassigned_callable_alias(&name);
    }
}

fn visit_catch_clause_with_scope<'a>(collector: &mut ImportCollector, clause: &CatchClause<'a>) {
    let pushed = collector.push_lexical_scope();
    if let Some(param) = &clause.param {
        collector.add_binding_names(&param.pattern);
    }
    walk::walk_catch_clause(collector, clause);
    collector.pop_lexical_scope(pushed);
}
