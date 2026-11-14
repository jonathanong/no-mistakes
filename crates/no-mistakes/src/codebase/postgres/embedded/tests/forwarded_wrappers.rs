use super::super::TrustedSqlTag;
use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

fn calls(case: &str, options: &EmbeddedSqlOptions) -> super::super::EmbeddedSqlFileFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/issue-3351")
        .join(case)
        .join("index.ts");
    extract_embedded_sql_from_source(&path, &std::fs::read_to_string(&path).unwrap(), options)
}

#[test]
fn direct_forwarding_preserves_query_arguments() {
    let facts = calls("forwarding", &EmbeddedSqlOptions::configured("pg", &[]));
    let found: Vec<_> = facts
        .calls
        .iter()
        .map(|call| (call.callee.as_str(), call.sql_text.as_deref()))
        .collect();
    assert_eq!(
        found,
        [
            ("same", Some("SELECT 1")),
            ("renamed", Some("SELECT 2")),
            ("extra", Some("SELECT 3")),
            ("functionForm", Some("SELECT 4")),
            ("one", Some("SELECT 5")),
            ("same", Some("SELECT 6 OFFSET 1")),
            ("blockArrow", Some("SELECT 7")),
            ("blockFunction", Some("SELECT 8")),
        ]
    );
}

#[test]
fn non_forwarding_wrappers_remain_unresolved() {
    let facts = calls("non-forwarding", &EmbeddedSqlOptions::configured("pg", &[]));
    for name in [
        "transformed",
        "reordered",
        "dropped",
        "spread",
        "defaulted",
        "shadowed",
        "mutable",
        "droppedMiddle",
        "namedShadow",
        "awaited",
        "withEffect",
    ] {
        assert!(
            !facts
                .calls
                .iter()
                .any(|call| call.callee == name && call.sql_text.is_some()),
            "{name}: {facts:?}"
        );
    }
    for line in [2, 3, 4, 5, 6, 8, 9, 11, 12] {
        assert!(
            facts
                .calls
                .iter()
                .any(|call| call.line == line && call.is_unanalyzable()),
            "line {line}: {facts:?}"
        );
    }
}

#[test]
fn configured_transaction_import_forwards_sql_and_binds() {
    let facts = calls(
        "transaction",
        &EmbeddedSqlOptions::configured("db-lib", &["transaction".into()]).with_trusted_sql_tags(
            &[TrustedSqlTag {
                module: "db-lib".into(),
                name: "sql".into(),
            }],
        ),
    );
    let found: Vec<_> = facts
        .calls
        .iter()
        .map(|call| (call.callee.as_str(), call.sql_text.as_deref()))
        .collect();
    assert_eq!(
        found,
        [
            ("q", Some("UPDATE items SET a = 1 WHERE id = $1")),
            ("withExtra", Some("UPDATE items SET a = 2 WHERE id = $1")),
        ]
    );
}

#[test]
fn duplicate_function_parameter_cannot_prove_forwarding() {
    let facts = calls("duplicate", &EmbeddedSqlOptions::configured("pg", &[]));
    assert!(
        facts
            .calls
            .iter()
            .any(|call| call.line == 2 && call.is_unanalyzable()),
        "{facts:?}"
    );
    assert!(
        !facts
            .calls
            .iter()
            .any(|call| call.callee == "duplicate" && call.sql_text.is_some()),
        "{facts:?}"
    );
}

#[test]
fn earlier_call_keeps_unknown_executor_obligation() {
    for case in ["early-call", "early-alias", "early-member"] {
        let facts = calls(case, &EmbeddedSqlOptions::configured("pg", &[]));
        let line = if case == "early-alias" { 6 } else { 5 };
        assert!(
            facts
                .calls
                .iter()
                .any(|call| call.line == line && call.is_unanalyzable()),
            "{case}: {facts:?}"
        );
    }
}
