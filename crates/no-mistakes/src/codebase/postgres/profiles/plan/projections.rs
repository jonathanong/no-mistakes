use super::*;

pub(super) fn configure(
    config: &NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) {
    plan.postgres_grouping |= grouping_requested(config);
    plan.postgres_bounds |= !config
        .rule_applications("postgres-bounded-statements")
        .is_empty();
    configure_body_bounds(config, plan);
    plan.postgres_index_probes |= !config
        .rule_applications("postgres-required-predicates")
        .is_empty();
    plan.postgres_json_writes |= config
        .rule_applications("postgres-json-relation-ids")
        .iter()
        .any(|rule| rule.options.get("idKeyPattern").is_some());
    plan.postgres_fragments |= !config
        .rule_applications("postgres-sql-shape-policy")
        .is_empty();
}

pub(crate) fn grouping_requested(config: &NoMistakesConfig) -> bool {
    config
        .rule_applications("postgres-grouping-not-aggregation")
        .iter()
        .any(|rule| {
            rule.options
                .get("schemaCatalogPath")
                .and_then(serde_yaml::Value::as_str)
                .is_some_and(|path| !path.trim().is_empty())
        })
}

#[cfg(test)]
mod tests;
