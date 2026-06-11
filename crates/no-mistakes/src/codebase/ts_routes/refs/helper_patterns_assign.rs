fn apply_helper_assignment_expression<'a>(
    assignment: &'a oxc::ast::ast::AssignmentExpression<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &mut HashMap<String, Vec<String>>,
    depth: usize,
) {
    let oxc::ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left else {
        return;
    };
    let value = evaluate_route_expression(&assignment.right, defs, imported_helpers, env, depth + 1);
    if assignment.operator == oxc::ast::ast::AssignmentOperator::Addition {
        let current = env
            .get(ident.name.as_str())
            .cloned()
            .unwrap_or_else(|| vec!["*".to_string()]);
        env.insert(ident.name.to_string(), concat_candidates(&current, &value));
    } else if assignment.operator == oxc::ast::ast::AssignmentOperator::Assign {
        env.insert(ident.name.to_string(), value);
    }
}

fn merge_helper_env(
    env: &mut HashMap<String, Vec<String>>,
    branch_env: HashMap<String, Vec<String>>,
) {
    for (name, values) in branch_env {
        let mut merged = env.remove(&name).unwrap_or_default();
        merged.extend(values);
        env.insert(name, dedupe_candidates(merged));
    }
}

fn replace_helper_env_with_branches(
    env: &mut HashMap<String, Vec<String>>,
    branch_envs: Vec<HashMap<String, Vec<String>>>,
) {
    let mut merged = HashMap::<String, Vec<String>>::new();
    for branch_env in branch_envs {
        for (name, values) in branch_env {
            merged.entry(name).or_default().extend(values);
        }
    }
    for values in merged.values_mut() {
        *values = dedupe_candidates(std::mem::take(values));
    }
    *env = merged;
}
