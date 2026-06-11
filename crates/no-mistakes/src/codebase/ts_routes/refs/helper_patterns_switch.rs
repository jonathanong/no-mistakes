fn evaluate_helper_switch_statement<'a>(
    switch_stmt: &'a oxc::ast::ast::SwitchStatement<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) -> HelperStatementEvaluation {
    let mut values = Vec::new();
    let base_env = env.clone();
    let mut case_envs = Vec::new();
    let mut fallthrough_envs = Vec::new();
    let has_default = switch_stmt.cases.iter().any(|case| case.test.is_none());
    for (case_index, case) in switch_stmt.cases.iter().enumerate() {
        let mut entry_envs = vec![base_env.clone()];
        entry_envs.append(&mut fallthrough_envs);
        let mut next_fallthrough_envs = Vec::new();
        let is_last_case = case_index + 1 == switch_stmt.cases.len();
        for mut case_env in entry_envs {
            let mut exits_switch = false;
            let mut case_can_continue = true;
            for stmt in &case.consequent {
                if matches!(stmt, Statement::BreakStatement(_)) {
                    exits_switch = true;
                    break;
                }
                let evaluation = evaluate_helper_return_statement(
                    stmt,
                    defs,
                    imported_helpers,
                    &mut case_env,
                    depth + 1,
                );
                values.extend(evaluation.values);
                if !evaluation.can_continue {
                    case_can_continue = false;
                    break;
                }
            }
            if !case_can_continue {
                continue;
            }
            if exits_switch || is_last_case {
                case_envs.push(case_env);
            } else {
                next_fallthrough_envs.push(case_env);
            }
        }
        fallthrough_envs = next_fallthrough_envs;
    }
    let can_continue = !has_default || !case_envs.is_empty();
    if !case_envs.is_empty() {
        if has_default {
            replace_helper_env_with_branches(env, case_envs);
        } else {
            let mut merged_env = base_env;
            for case_env in case_envs {
                merge_helper_env(&mut merged_env, case_env);
            }
            *env = merged_env;
        }
    }
    HelperStatementEvaluation {
        values,
        can_continue,
    }
}
