use super::{effective_sql_include, profile_sql_include, PostgresSchemaOptions};

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
