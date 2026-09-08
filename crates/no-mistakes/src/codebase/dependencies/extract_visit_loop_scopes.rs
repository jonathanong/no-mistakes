fn visit_for_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &ForStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    walk::walk_for_statement(collector, statement);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_in_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &ForInStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    walk::walk_for_in_statement(collector, statement);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_of_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &ForOfStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    walk::walk_for_of_statement(collector, statement);
    collector.pop_lexical_scope(pushed);
}

fn visit_switch_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &SwitchStatement<'a>,
) {
    collector.visit_expression(&statement.discriminant);
    let pushed = collector.push_lexical_scope();
    for case in &statement.cases {
        predeclare_lexical_declarations(collector, &case.consequent);
    }
    collector.visit_switch_cases(&statement.cases);
    collector.pop_lexical_scope(pushed);
}
