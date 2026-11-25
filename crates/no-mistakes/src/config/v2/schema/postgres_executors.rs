use super::RuleDef;
use anyhow::Result;
use serde_yaml::Value;

pub(crate) const RULE_IDS: &[&str] = &[
    "postgres-no-test-state-mutation",
    "postgres-bounded-statements",
    "postgres-explicit-columns",
    "postgres-generated-column-predicates",
    "postgres-no-generated-column-writes",
    "postgres-required-predicates",
    "postgres-sql-shape-policy",
    "postgres-no-offset",
    "postgres-no-duplicate-keyed-lookups",
    "postgres-no-transformed-uuid-predicates",
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
    "postgres-conflict-ordering",
    "postgres-lock-written-rows-only",
    "postgres-lock-inventory",
    "postgres-lock-reason",
    "postgres-lock-ordering",
    "postgres-idempotent-insert",
    "postgres-require-query-annotation",
];

pub(crate) const ALL_RULE_IDS: &[&str] = &[
    "postgres-no-test-state-mutation",
    "postgres-bounded-statements",
    "postgres-explicit-columns",
    "postgres-generated-column-predicates",
    "postgres-no-generated-column-writes",
    "postgres-required-predicates",
    "postgres-sql-shape-policy",
    "postgres-no-offset",
    "postgres-no-duplicate-keyed-lookups",
    "postgres-no-transformed-uuid-predicates",
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
    "postgres-conflict-ordering",
    "postgres-lock-written-rows-only",
    "postgres-lock-inventory",
    "postgres-lock-reason",
    "postgres-lock-ordering",
    "postgres-idempotent-insert",
    "postgres-require-query-annotation",
    "postgres-sql-statement-policy",
    "postgres-json-relation-ids",
    "postgres-grouping-not-aggregation",
];

pub(super) fn validate(rule: &RuleDef) -> Result<()> {
    if ALL_RULE_IDS.contains(&rule.rule.as_str())
        && rule.options.get("executorNames").is_some()
        && rule.options.get("extraExecutorNames").is_some()
    {
        anyhow::bail!("{} options executorNames and extraExecutorNames cannot both be set; use extraExecutorNames to extend the defaults", rule.rule);
    }
    if !RULE_IDS.contains(&rule.rule.as_str()) {
        return Ok(());
    }
    // Inspect the raw options: deserialized defaults erase an explicit empty list.
    let import = rule.options.get("importSpecifier");
    let names = rule.options.get("executorNames");
    let extras = rule.options.get("extraExecutorNames");
    let helpers = rule.options.get("helpers");
    if matches!(
        rule.rule.as_str(),
        "postgres-lock-inventory" | "postgres-lock-reason"
    ) && helpers.is_some_and(|value| {
        !value.is_sequence()
            || value
                .as_sequence()
                .is_some_and(|helpers| !helpers.is_empty())
    }) {
        return Ok(());
    }
    if import.is_some_and(|value| value.as_str().is_some_and(|s| !s.is_empty()))
        || names.is_some_and(Value::is_sequence)
        || extras.is_some_and(Value::is_sequence)
    {
        return Ok(());
    }
    // Let the ordinary option deserializer diagnose malformed supplied shapes.
    if import.is_some_and(|value| !value.is_null() && !value.is_string())
        || names.is_some_and(|value| !value.is_null())
        || extras.is_some_and(|value| !value.is_null())
    {
        return Ok(());
    }
    anyhow::bail!(
        "{} option importSpecifier: set importSpecifier (or executorNames) to select executor calls; set executorNames: [] to scan only SQL files and native SQL (see docs/migrations/explicit-postgres-executors.md)",
        rule.rule
    );
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

/// Filesystem rules whose configured executor projections must be declared at
/// the request boundary before TS/JS fact collection begins.
#[doc(hidden)]
pub const PREPARED_EMBEDDED_SQL_RULE_IDS: &[&str] = &[
    "postgres-idempotent-insert",
    "postgres-conflict-ordering",
    "postgres-lock-written-rows-only",
    "postgres-lock-inventory",
    "postgres-lock-reason",
    "postgres-lock-ordering",
    "postgres-required-predicates",
    "postgres-generated-column-predicates",
    "postgres-explicit-columns",
    "postgres-no-test-state-mutation",
    "postgres-bounded-statements",
    "postgres-no-offset",
    "postgres-no-transformed-uuid-predicates",
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
    "postgres-sql-shape-policy",
    "postgres-sql-statement-policy",
    "postgres-no-generated-column-writes",
    "postgres-require-query-annotation",
    "postgres-json-relation-ids",
    "postgres-no-duplicate-keyed-lookups",
    "postgres-grouping-not-aggregation",
];
