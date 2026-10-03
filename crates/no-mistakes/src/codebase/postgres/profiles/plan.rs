use super::*;

/// Declare SQL projections before the shared request fact pass.
pub fn configure_prepared_postgres_plan(
    config: &NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> Result<()> {
    let dml_rules = [
        "postgres-required-predicates",
        "postgres-no-offset",
        "postgres-generated-column-predicates",
        "postgres-explicit-columns",
        "postgres-bounded-statements",
        "postgres-sql-shape-policy",
        "postgres-no-generated-column-writes",
    ];
    plan.postgres_dml |= dml_rules
        .iter()
        .any(|id| !config.rule_applications(id).is_empty());
    plan.postgres_bounds |= !config
        .rule_applications("postgres-bounded-statements")
        .is_empty();
    plan.postgres_fragments |= !config
        .rule_applications("postgres-sql-shape-policy")
        .is_empty();
    let schema_rules = [
        "postgres-generated-column-predicates",
        "postgres-no-generated-column-writes",
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
    plan.postgres_write_sql_include
        .extend(write_patterns(config)?);
    plan.postgres_write_sql_include.sort();
    plan.postgres_write_sql_include.dedup();
    plan.postgres_sql_include.sort();
    plan.postgres_sql_include.dedup();
    Ok(())
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct WriteOptions {
    include: Vec<String>,
}

pub(super) fn write_patterns(config: &NoMistakesConfig) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    for rule in config.rule_applications("postgres-no-generated-column-writes") {
        let options: WriteOptions = rule.try_rule_options()?;
        patterns.extend(if options.include.is_empty() {
            vec!["**/*.sql".into()]
        } else {
            options.include
        });
    }
    Ok(patterns)
}
