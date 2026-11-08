fn configure_body_bounds(config: &NoMistakesConfig, plan: &mut crate::codebase::check_facts::CheckFactPlan) {
    plan.postgres_body_bounds |= body_bounds_requested(config);
}

pub(super) fn body_bounds_requested(config: &NoMistakesConfig) -> bool {
    config
        .rule_applications("postgres-bounded-statements")
        .iter()
        .any(|rule| {
            rule.options
                .get("materializedBodies")
                .and_then(serde_yaml::Value::as_str)
                .is_some_and(|value| value != "ignore")
        })
}
