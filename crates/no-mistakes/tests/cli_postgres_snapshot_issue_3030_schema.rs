use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::Command;

fn fixture(rule: &str, name: &str) -> (PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/postgres/issue-3030-schema")
        .join(rule)
        .join(name);
    let config = root.join(".no-mistakes.yml");
    (root, config)
}

#[test]
fn issue_3030_array_allow_snapshot() {
    let rule = "postgres-array-columns";
    let cases = [
        ("valid_column", "issue-3030-valid-column.yml"),
        ("definite_finding", "catalog.yml"),
    ]
    .map(|(label, config)| {
        (
            label,
            rules_path(rule, "fixture", "orders-tag-names"),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_issue_3030_array_allow", &cases);
}

#[test]
fn issue_3030_array_invalid_option_snapshot() {
    let root = rules_path("postgres-array-columns", "fixture", "orders-tag-names");
    let config = rules_path(
        "postgres-array-columns",
        "configs",
        "issue-3030-invalid-kind.yml",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    insta::assert_snapshot!(
        "postgres_issue_3030_array_invalid_option",
        format!("exitCode: 2\n{}", String::from_utf8_lossy(&output.stderr))
    );
}

#[test]
fn issue_3030_schema_proofs_snapshot() {
    let cases = [
        (
            "constraint_cross_schema",
            "postgres-constraint-validate",
            "issue-3030-cross-schema",
        ),
        (
            "constraint_matched_schema",
            "postgres-constraint-validate",
            "issue-3030-matched-schema",
        ),
        (
            "generated_case",
            "postgres-no-generated-column-writes",
            "issue-3030-case",
        ),
        (
            "generated_chronology",
            "postgres-no-generated-column-writes",
            "issue-3030-chronology",
        ),
        (
            "generated_quoted_trigger",
            "postgres-no-generated-column-writes",
            "issue-3030-quoted-trigger",
        ),
        (
            "generated_unquoted_trigger",
            "postgres-no-generated-column-writes",
            "issue-3030-unquoted-trigger",
        ),
        (
            "generated_quoted_positional",
            "postgres-no-generated-column-writes",
            "issue-3030-quoted-positional",
        ),
        (
            "generated_ordinary_positional",
            "postgres-no-generated-column-writes",
            "issue-3030-ordinary-positional",
        ),
        (
            "redundant_drop",
            "postgres-redundant-index",
            "issue-3030-same-line-drop",
        ),
        (
            "redundant_live",
            "postgres-redundant-index",
            "issue-3030-live",
        ),
        (
            "conflict_ties",
            "postgres-conflict-ordering",
            "issue-3030-ties",
        ),
        (
            "conflict_only",
            "postgres-conflict-ordering",
            "issue-3030-only",
        ),
        (
            "bounded_roots",
            "postgres-bounded-statements",
            "issue-3030-roots",
        ),
        (
            "bounded_embedded",
            "postgres-bounded-statements",
            "issue-3030-embedded",
        ),
    ];
    let cases = cases.map(|(label, rule, name)| {
        let (root, config) = fixture(rule, name);
        (label, root, config)
    });
    assert_saved_fixture("postgres_issue_3030_schema_proofs", &cases);
}
