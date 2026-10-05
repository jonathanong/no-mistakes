use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/conflict-ordering/cases")
            .join(scenario),
    )
}

fn check(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn postgres_conflict_ordering_cli_reports_missing_canonical_order() {
    let output = check(&fixture("fail-missing-order"));
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1: {body}");
    assert!(body.contains("postgres-conflict-ordering"), "{body}");
    assert!(body.contains("missing-canonical-order"), "{body}");
}

#[test]
fn postgres_conflict_ordering_cli_accepts_a_generated_text_expression_key() {
    // Regression: a catalog generated from real PostgreSQL reports `orderingSupported: true`
    // for a `lower(text)` key, so the documented expression arbiter resolves.
    let output = check(&fixture("pass-expression-text-key"));
    assert!(
        output.status.success(),
        "exit non-zero: {}",
        stdout(&output)
    );
}

#[test]
fn postgres_conflict_ordering_cli_normalizes_redundant_predicate_parentheses() {
    // Regression: the catalog stores `((a) AND (b))`; the writer's unparenthesized predicate
    // must resolve to the same partial index.
    let output = check(&fixture("pass-partial-unparenthesized"));
    assert!(
        output.status.success(),
        "exit non-zero: {}",
        stdout(&output)
    );
}

#[test]
fn postgres_conflict_ordering_cli_keeps_conjunct_order_significant() {
    let output = check(&fixture("fail-partial-reordered-conjuncts"));
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1: {body}");
    assert!(body.contains("unresolved-arbiter"), "{body}");
}

#[test]
fn postgres_conflict_ordering_cli_accepts_provably_single_row_sources() {
    // Regression: a FROM-less SELECT, a unique-key lookup and LIMIT 1 yield one row.
    let output = check(&fixture("pass-single-row-source"));
    assert!(
        output.status.success(),
        "exit non-zero: {}",
        stdout(&output)
    );
}

#[test]
fn postgres_conflict_ordering_cli_keeps_single_row_lookalikes_failing_closed() {
    let output = check(&fixture("fail-single-row-lookalikes"));
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1: {body}");
    assert_eq!(
        body.matches("\"target\":\"missing-canonical-order\"")
            .count(),
        8,
        "{body}"
    );
    assert_eq!(
        body.matches("\"target\":\"noncanonical-order\"").count(),
        1,
        "{body}"
    );
}

#[test]
fn postgres_conflict_ordering_cli_maps_positional_and_constant_order_keys() {
    let output = check(&fixture("pass-order-by-forms"));
    assert!(
        output.status.success(),
        "exit non-zero: {}",
        stdout(&output)
    );
}

#[test]
fn postgres_conflict_ordering_cli_accepts_a_catalog_ordered_writer() {
    let output = check(&fixture("pass-sql-include"));
    assert!(
        output.status.success(),
        "exit non-zero: {}",
        stdout(&output)
    );
}
