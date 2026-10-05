use super::EmbeddedSqlOptions;

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn neither_option_is_a_configuration_error_naming_the_rule() {
    for (specifier, executors) in [(None, None), (Some(""), None)] {
        let error = EmbeddedSqlOptions::for_rule("postgres-lock-ordering", specifier, executors)
            .expect_err("no executor selection must be rejected");
        assert_eq!(
            error.to_string(),
            "postgres-lock-ordering option importSpecifier: set importSpecifier (or \
executorNames) to select executor calls; set executorNames: [] to scan only SQL files \
and native SQL (see docs/migrations/explicit-postgres-executors.md)"
        );
    }
}

#[test]
fn error_message_uses_the_supplied_rule_id() {
    let error = EmbeddedSqlOptions::for_rule("postgres-no-offset", None, None).unwrap_err();
    assert!(error
        .to_string()
        .starts_with("postgres-no-offset option importSpecifier:"));
}

#[test]
fn explicit_empty_executor_names_opt_out_selects_no_executors() {
    // Absent and empty differ: `executorNames: []` is deliberate.
    let options = EmbeddedSqlOptions::for_rule("postgres-lock-ordering", None, Some(&[])).unwrap();
    assert_eq!(options, EmbeddedSqlOptions::default());
    let blank = EmbeddedSqlOptions::for_rule("postgres-lock-ordering", Some(""), Some(&[]));
    assert_eq!(blank.unwrap(), EmbeddedSqlOptions::default());
}

#[test]
fn import_specifier_alone_enables_the_standard_executor_names() {
    let options =
        EmbeddedSqlOptions::for_rule("postgres-lock-ordering", Some("@example/db"), None).unwrap();
    assert_eq!(options.import_specifier, "@example/db");
    assert_eq!(options.executor_names, ["query", "read", "write"]);
}

#[test]
fn import_specifier_with_empty_names_keeps_the_standard_executor_names() {
    // Preserves the meaning configs had before absent and empty were
    // distinguished: a module with no names uses query, read and write.
    let options =
        EmbeddedSqlOptions::for_rule("postgres-lock-ordering", Some("@example/db"), Some(&[]))
            .unwrap();
    assert_eq!(options.executor_names, ["query", "read", "write"]);
}

#[test]
fn explicit_names_are_sorted_and_deduplicated_with_or_without_a_module() {
    let selected = names(&["write", "read", "write"]);
    let anywhere =
        EmbeddedSqlOptions::for_rule("postgres-lock-ordering", None, Some(&selected)).unwrap();
    assert_eq!(anywhere.import_specifier, "");
    assert_eq!(anywhere.executor_names, ["read", "write"]);
    let scoped = EmbeddedSqlOptions::for_rule(
        "postgres-lock-ordering",
        Some("@example/db"),
        Some(&selected),
    )
    .unwrap();
    assert_eq!(scoped.import_specifier, "@example/db");
    assert_eq!(scoped.executor_names, ["read", "write"]);
}
