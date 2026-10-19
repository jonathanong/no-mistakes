use super::PostgresSchemaOptions;

/// SQL globs after the empty-list default.
///
/// An unlocked empty list still becomes `**/*.sql`. A list locked by project
/// scope stays as written, including when that list is empty.
pub(crate) fn effective_sql_include(patterns: &[String], locked: bool) -> Vec<String> {
    if locked || !patterns.is_empty() {
        patterns.to_vec()
    } else {
        PostgresSchemaOptions::default().sql_include
    }
}

/// Profile aggregation. Three rules already keep an empty list (no SQL sources
/// or no extra SQL filter) even when the list is not locked.
pub(crate) fn profile_sql_include(
    rule_id: &str,
    patterns: Vec<String>,
    locked: bool,
) -> Vec<String> {
    if locked || !patterns.is_empty() || keeps_empty_sql_include(rule_id) {
        patterns
    } else {
        PostgresSchemaOptions::default().sql_include
    }
}

fn keeps_empty_sql_include(rule_id: &str) -> bool {
    matches!(
        rule_id,
        "postgres-explicit-columns" | "postgres-bounded-statements" | "postgres-no-offset"
    )
}

#[cfg(test)]
mod tests;
