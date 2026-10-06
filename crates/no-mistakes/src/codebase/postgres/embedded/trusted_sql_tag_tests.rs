use super::{extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions, TrustedSqlTag};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn tags(module: &str, name: &str) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
        module: module.to_string(),
        name: name.to_string(),
    }])
}

fn calls(name: &str, options: &EmbeddedSqlOptions) -> Vec<super::EmbeddedSqlCall> {
    let path = fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(&path, &source, options).calls
}

fn kind(name: &str, options: &EmbeddedSqlOptions) -> EmbeddedSqlKind {
    let calls = calls(name, options);
    assert_eq!(calls.len(), 1, "{name}: {calls:?}");
    calls[0].kind
}

#[test]
fn named_import_is_analyzed_only_when_the_option_is_present() {
    let absent = EmbeddedSqlOptions::configured("@example/db", &[]);
    assert_eq!(
        kind("trusted-sql-tag-named.ts", &absent),
        EmbeddedSqlKind::Dynamic
    );
    let calls = calls("trusted-sql-tag-named.ts", &tags("@example/db", "sql"));
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("\n    SELECT id\n    FROM documents\n    WHERE account_id = sql_placeholder_1\n  ")
    );
}

#[test]
fn renamed_local_binding_is_trusted() {
    let calls = calls("trusted-sql-tag-renamed.ts", &tags("@example/db", "sql"));
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Inline);
    assert!(
        calls[0]
            .sql_text
            .as_deref()
            .unwrap()
            .contains("sql_placeholder_1"),
        "{calls:?}"
    );
}

#[test]
fn subpath_matches_and_a_sibling_prefix_does_not() {
    let options = tags("@example/db", "sql");
    assert_eq!(
        kind("trusted-sql-tag-subpath.ts", &options),
        EmbeddedSqlKind::Inline
    );
    assert_eq!(
        kind("trusted-sql-tag-sibling.ts", &options),
        EmbeddedSqlKind::Dynamic
    );
}

#[test]
fn other_module_default_import_and_wrong_export_stay_untrusted() {
    let options = tags("@example/db", "sql");
    for name in [
        "trusted-sql-tag-other-module.ts",
        "trusted-sql-tag-default.ts",
        "trusted-sql-tag-wrong-export.ts",
    ] {
        assert_eq!(kind(name, &options), EmbeddedSqlKind::Dynamic, "{name}");
    }
}

#[test]
fn shadowed_local_and_type_only_import_fail_closed() {
    let options = tags("@example/db", "sql");
    assert_eq!(
        kind("trusted-sql-tag-shadowed.ts", &options),
        EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        kind("trusted-sql-tag-type-only.ts", &options),
        EmbeddedSqlKind::Dynamic
    );
}

#[test]
fn empty_module_or_name_matches_nothing() {
    assert_eq!(
        kind("trusted-sql-tag-named.ts", &tags("", "sql")),
        EmbeddedSqlKind::Dynamic
    );
    assert_eq!(
        kind("trusted-sql-tag-named.ts", &tags("@example/db", "")),
        EmbeddedSqlKind::Dynamic
    );
}
