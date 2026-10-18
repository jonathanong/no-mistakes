use super::{fail_unanalyzable_sql, unanalyzable_sql_mode, UnanalyzableSqlMode};

#[test]
fn empty_and_fail_are_fail_closed() {
    assert!(fail_unanalyzable_sql("rule", "").unwrap());
    assert!(fail_unanalyzable_sql("rule", "fail").unwrap());
    assert!(fail_unanalyzable_sql("rule", "FAIL").unwrap());
}

#[test]
fn ignore_disables_fail_closed() {
    assert!(!fail_unanalyzable_sql("rule", "ignore").unwrap());
    assert!(!fail_unanalyzable_sql("rule", " Ignore ").unwrap());
}

#[test]
fn advise_is_non_failing_but_distinct_from_ignore() {
    assert!(!fail_unanalyzable_sql("rule", " Advise ").unwrap());
    assert_eq!(
        unanalyzable_sql_mode("rule", "advise").unwrap(),
        UnanalyzableSqlMode::Advise
    );
    assert_eq!(
        unanalyzable_sql_mode("rule", "ignore").unwrap(),
        UnanalyzableSqlMode::Ignore
    );
}

#[test]
fn unknown_mode_is_a_config_error() {
    let error = fail_unanalyzable_sql("postgres-idempotent-insert", "fial").unwrap_err();
    assert!(error.to_string().contains("unanalyzableSql"), "{error}");
}
