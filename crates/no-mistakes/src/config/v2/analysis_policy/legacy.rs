use super::*;

pub(super) fn legacy_policy(rule: &RuleDef) -> Result<Option<Unanalyzable>> {
    if !sql_policy_rule(&rule.rule) {
        if legacy_sql_options(&rule.rule) {
            // Project scope historically copied this key to rules that ignored
            // it. Those opaque payloads must remain valid at maximum effort.
            return Ok(rule
                .options
                .get("unanalyzableSql")
                .and_then(|value| serde_yaml::from_value(value.clone()).ok()));
        }
        return option(rule, "unanalyzableSql");
    }
    let Some(value) = rule
        .options
        .get("unanalyzableSql")
        .and_then(serde_yaml::Value::as_str)
    else {
        return Ok(None);
    };
    let mode = if rule.rule == "postgres-require-query-annotation" {
        match value {
            "report" => Some(Unanalyzable::Report),
            "advise" => Some(Unanalyzable::Advise),
            "ignore" => Some(Unanalyzable::Ignore),
            _ => None,
        }
    } else {
        match value.trim().to_ascii_lowercase().as_str() {
            "fail" | "" => Some(Unanalyzable::Report),
            "advise" => Some(Unanalyzable::Advise),
            "ignore" => Some(Unanalyzable::Ignore),
            _ => None,
        }
    };
    Ok(mode)
}

pub(crate) fn validate_low_alias(rule: &RuleDef) -> Result<()> {
    if rule.options.get("unanalyzableSql").is_some() && legacy_policy(rule)?.is_none() {
        anyhow::bail!("{}: invalid options.unanalyzableSql", rule.rule);
    }
    Ok(())
}

pub(in super::super) fn legacy_sql_options(rule: &str) -> bool {
    rule.starts_with("postgres-")
}

pub(in super::super) fn sql_policy_rule(rule: &str) -> bool {
    matches!(
        rule,
        "postgres-bounded-statements"
            | "postgres-conflict-ordering"
            | "postgres-explicit-columns"
            | "postgres-generated-column-predicates"
            | "postgres-idempotent-insert"
            | "postgres-lock-ordering"
            | "postgres-no-generated-column-writes"
            | "postgres-no-offset"
            | "postgres-require-query-annotation"
            | "postgres-required-predicates"
            | "postgres-sql-shape-policy"
            | "postgres-sql-statement-policy"
    )
}
