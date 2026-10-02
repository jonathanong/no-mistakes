use super::*;

/// Declare SQL projections before the shared request fact pass.
pub fn configure_prepared_postgres_plan(
    config: &NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let dml_rules = ["postgres-required-predicates", "postgres-generated-column-predicates"];
    plan.postgres_dml |= dml_rules
        .iter()
        .any(|id| !config.rule_applications(id).is_empty());
    let schema_rules = [
        "postgres-generated-column-predicates",
        "postgres-identifier-length",
        "postgres-no-add-column",
        "postgres-require-named-constraints",
        "postgres-redundant-index",
        "postgres-fk-index",
        "postgres-constraint-validate",
        "postgres-sql-statement-policy",
        "postgres-require-fk-on-delete",
    ];
    plan.postgres_schema |= schema_rules
        .iter()
        .any(|id| !config.rule_applications(id).is_empty());
    plan.postgres_sql_include
        .extend(sql_patterns(config, &schema_rules)?);
    plan.postgres_sql_include
        .extend(sql_patterns(config, &dml_rules)?);
    plan.postgres_sql_include.sort();
    plan.postgres_sql_include.dedup();
    Ok(())
}
