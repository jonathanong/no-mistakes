fn evaluate_helper_switch_statement<'a>(
    switch_stmt: &'a oxc::ast::ast::SwitchStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let mut values = Vec::new();
    let base_env = env.clone();
    let mut case_envs = Vec::new();
    for case in &switch_stmt.cases {
        let mut case_env = base_env.clone();
        for stmt in &case.consequent {
            values.extend(evaluate_helper_return_statement(
                stmt,
                defs,
                imported_helpers,
                &mut case_env,
                depth + 1,
            ));
        }
        case_envs.push(case_env);
    }
    if !case_envs.is_empty() {
        let mut merged_env = base_env;
        for case_env in case_envs {
            merge_helper_env(&mut merged_env, case_env);
        }
        *env = merged_env;
    }
    values
}
