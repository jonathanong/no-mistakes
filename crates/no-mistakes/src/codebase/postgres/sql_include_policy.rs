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
mod tests {
    use super::*;

    #[test]
    fn unlocked_empty_list_uses_the_schema_default() {
        assert_eq!(
            effective_sql_include(&[], false),
            PostgresSchemaOptions::default().sql_include
        );
        assert_eq!(
            profile_sql_include("postgres-identifier-length", Vec::new(), false),
            PostgresSchemaOptions::default().sql_include
        );
    }

    #[test]
    fn locked_empty_list_stays_empty() {
        assert!(effective_sql_include(&[], true).is_empty());
        assert!(profile_sql_include("postgres-identifier-length", Vec::new(), true).is_empty());
        assert!(profile_sql_include("postgres-no-offset", Vec::new(), false).is_empty());
    }

    #[test]
    fn explicit_patterns_are_kept() {
        let patterns = vec!["migrations/**/*.sql".to_string()];
        assert_eq!(effective_sql_include(&patterns, false), patterns);
        assert_eq!(
            profile_sql_include("postgres-no-offset", patterns.clone(), true),
            patterns
        );
    }
}
