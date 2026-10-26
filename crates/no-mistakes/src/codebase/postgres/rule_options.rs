use anyhow::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnanalyzableSqlMode {
    Fail,
    Ignore,
    Advise,
}

pub fn unanalyzable_sql_mode(rule: &str, value: &str) -> Result<UnanalyzableSqlMode> {
    match value.trim() {
        "" => Ok(UnanalyzableSqlMode::Fail),
        other if other.eq_ignore_ascii_case("fail") => Ok(UnanalyzableSqlMode::Fail),
        other if other.eq_ignore_ascii_case("ignore") => Ok(UnanalyzableSqlMode::Ignore),
        other if other.eq_ignore_ascii_case("advise") => Ok(UnanalyzableSqlMode::Advise),
        other => anyhow::bail!(
            "{rule}: unanalyzableSql must be `fail`, `ignore`, or `advise`, got `{other}`"
        ),
    }
}

/// Parse `unanalyzableSql`: empty/`fail` fail closed, `ignore`/`advise` do not.
pub fn fail_unanalyzable_sql(rule: &str, value: &str) -> Result<bool> {
    Ok(unanalyzable_sql_mode(rule, value)? == UnanalyzableSqlMode::Fail)
}

/// The request needs a report-local proof sink when any effective SQL rule
/// application asks for non-failing residual diagnostics.
pub fn has_sql_advice(config: &crate::config::v2::NoMistakesConfig) -> bool {
    if [
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
    ]
    .iter()
    .flat_map(|id| config.rule_applications(id))
    .any(|rule| {
        config.effective_rule_analysis(rule).is_ok_and(|policy| {
            policy.effort == crate::config::v2::Effort::Low
                && policy.unanalyzable == crate::config::v2::Unanalyzable::Advise
        })
    }) {
        return true;
    }
    const RULES: &[&str] = &[
        "postgres-no-test-state-mutation",
        "postgres-bounded-statements",
        "postgres-conflict-ordering",
        "postgres-explicit-columns",
        "postgres-generated-column-predicates",
        "postgres-idempotent-insert",
        "postgres-lock-ordering",
        "postgres-no-generated-column-writes",
        "postgres-no-offset",
        "postgres-no-transformed-uuid-predicates",
        "postgres-require-query-annotation",
        "postgres-required-predicates",
        "postgres-sql-shape-policy",
        "postgres-sql-statement-policy",
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
    ];
    let key = serde_yaml::Value::String("unanalyzableSql".to_string());
    RULES.iter().any(|id| {
        config.rule_applications(id).iter().any(|rule| {
            rule.options
                .as_mapping()
                .and_then(|options| options.get(&key))
                .and_then(serde_yaml::Value::as_str)
                .is_some_and(|mode| mode.trim().eq_ignore_ascii_case("advise"))
        })
    })
}

#[cfg(test)]
mod tests;
