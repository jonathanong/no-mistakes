fn evaluate_helper_switch_statement<'a>(
    switch_stmt: &'a oxc::ast::ast::SwitchStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let mut values = Vec::new();
    for case in &switch_stmt.cases {
        for stmt in &case.consequent {
            values.extend(evaluate_helper_return_statement(
                stmt,
                defs,
                imported_helpers,
                env,
                depth + 1,
            ));
        }
    }
    values
}
