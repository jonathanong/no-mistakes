use super::tests::{config, fixture};
use super::{check_with_files, RULE_ID};

fn error(yaml: &str) -> String {
    check_with_files(&fixture(), &config(yaml), &[])
        .unwrap_err()
        .to_string()
}

#[test]
fn a_catalog_is_required_only_when_mapping_or_fingerprint_detection_is_set() {
    let expected = format!("{RULE_ID} option schemaCatalogPath: required when keyColumnMap or fingerprintNamePattern is set");
    for yaml in [
        "keyColumnMap: {accountId: account_id}",
        "fingerprintNamePattern: '_sha256$'",
        "schemaCatalogPath: ' '\nkeyColumnMap: {accountId: account_id}",
    ] {
        assert_eq!(error(yaml), expected);
    }
}

#[test]
fn empty_and_invalid_patterns_have_option_specific_errors() {
    assert_eq!(
        error("idKeyPattern: ''"),
        format!("{RULE_ID} option idKeyPattern: must not be empty")
    );
    for option in ["idKeyPattern", "fingerprintNamePattern"] {
        let actual = error(&format!("schemaCatalogPath: schema.json\n{option}: '['"));
        assert!(actual.starts_with(&format!("{RULE_ID} option {option}: ")));
        assert!(actual.contains("unclosed character class"), "{actual}");
    }
}

#[test]
fn mapped_keys_and_column_names_must_not_be_empty() {
    for (mapping, detail) in [
        ("{'': account_id}", "key must not be empty"),
        ("{' ': account_id}", "key must not be empty"),
        ("{accountId: ''}", "column for accountId must not be empty"),
        ("{accountId: ' '}", "column for accountId must not be empty"),
    ] {
        assert_eq!(
            error(&format!(
                "schemaCatalogPath: schema.json\nkeyColumnMap: {mapping}"
            )),
            format!("{RULE_ID} option keyColumnMap: {detail}")
        );
    }
}

#[test]
fn allows_need_a_reason_and_a_table_or_column_object() {
    for reason in ["''", "' '"] {
        assert_eq!(
            error(&format!(
                "allow: [{{object: 'table:orders', reason: {reason}}}]"
            )),
            format!("{RULE_ID} option allow: entry table:orders needs a reason")
        );
    }
    for object in [
        "orders",
        "table:",
        "column:orders",
        "column:orders.",
        "function:orders",
        "column:.details",
    ] {
        let actual = error(&format!("allow: [{{object: '{object}', reason: history}}]"));
        assert!(
            actual.starts_with(&format!("{RULE_ID} option allow:")),
            "{actual}"
        );
        assert!(actual.contains("invalid object ref"), "{actual}");
    }
}

#[test]
fn duplicate_keys_are_rejected_by_the_real_config_loader() {
    let root = fixture();
    for name in ["no-mistakes.duplicate.yml", "no-mistakes.duplicate.jsonc"] {
        let result = crate::config::v2::load_v2_config(&root, Some(&root.join(name)));
        let actual = result.unwrap_err().to_string();
        assert!(
            actual.contains("postgres-json-relation-ids option keyColumnMap:"),
            "{actual}"
        );
        assert!(actual.contains("duplicate key accountId"), "{actual}");
    }
}

#[test]
fn file_glob_validation_reports_invalid_patterns() {
    for option in ["include", "exclude", "sqlInclude"] {
        let actual = error(&format!("{option}: ['[']"));
        assert!(actual.contains("invalid glob"), "{actual}");
    }
}

#[test]
fn duplicate_allow_entries_are_configuration_errors() {
    let actual = error("allow: [{object: 'table:orders', reason: history}, {object: 'table:orders', reason: duplicate}]");
    assert_eq!(
        actual,
        format!("{RULE_ID} option allow: duplicate entry table:orders")
    );
}
