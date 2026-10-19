use super::*;
use no_mistakes::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-catalog")
        .join(name)
}

fn report(root: &Path, config: &str) -> CheckResults {
    run_all_with_sql_verdicts(
        root.to_path_buf(),
        Some(root.join(config)),
        None,
        false,
        false,
        true,
    )
    .unwrap()
}

fn at_line<'a>(
    root: &Path,
    rows: &'a [SqlObligationVerdict],
    line: usize,
    rule: &str,
    subcheck: &str,
) -> &'a SqlObligationVerdict {
    let file = "src/query.ts";
    let source = std::fs::read_to_string(root.join(file)).unwrap();
    let matches = rows
        .iter()
        .filter(|row| {
            row.file == file
                && row.rule == rule
                && row.subcheck == subcheck
                && source[..row.span.0 as usize]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1
                    == line
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "{file}:{line} {rule} {subcheck}");
    matches[0]
}

fn assert_row(
    root: &Path,
    rows: &[SqlObligationVerdict],
    line: usize,
    subcheck: &str,
    status: SqlVerdictStatus,
    reason: Option<&str>,
) {
    let row = at_line(root, rows, line, "postgres-required-predicates", subcheck);
    assert_eq!(row.status, status, "{row:?}");
    if let Some(reason) = reason {
        assert!(
            row.residual_reasons.iter().any(|found| found == reason),
            "{row:?}"
        );
    } else {
        assert!(row.residual_reasons.is_empty(), "{row:?}");
    }
}

#[test]
fn required_predicates_preserve_catalog_and_partition_evidence_per_call() {
    let root = fixture("catalog");
    let rows = report(&root, ".no-mistakes.yml").sql_verdicts;
    let partition = "partition-key";
    let owner = "relation:events:column:owner_id";
    assert_row(
        &root,
        &rows,
        3,
        partition,
        SqlVerdictStatus::Violation,
        None,
    );
    assert_row(&root, &rows, 3, owner, SqlVerdictStatus::Violation, None);
    assert_row(
        &root,
        &rows,
        4,
        partition,
        SqlVerdictStatus::ProvenClear,
        None,
    );
    assert_row(&root, &rows, 4, owner, SqlVerdictStatus::ProvenClear, None);
    // The unknown joined table can own an unqualified missing key.
    assert_row(
        &root,
        &rows,
        6,
        partition,
        SqlVerdictStatus::Abstain,
        Some("unknown-relation-owner"),
    );
    assert_row(
        &root,
        &rows,
        7,
        partition,
        SqlVerdictStatus::ProvenClear,
        None,
    );
    assert_row(
        &root,
        &rows,
        8,
        partition,
        SqlVerdictStatus::Abstain,
        Some("unknown-relation"),
    );
    assert_row(
        &root,
        &rows,
        8,
        "relation:missing_table:column:id",
        SqlVerdictStatus::Abstain,
        Some("unknown-relation"),
    );
    assert_row(
        &root,
        &rows,
        9,
        partition,
        SqlVerdictStatus::Abstain,
        Some("missing-partition-key-catalog"),
    );
    assert_row(
        &root,
        &rows,
        10,
        partition,
        SqlVerdictStatus::Abstain,
        Some("expression-partition-key"),
    );
}

#[test]
fn required_column_without_catalog_abstains_at_the_confirmed_executor() {
    let root = fixture("no-catalog");
    let rows = report(&root, ".no-mistakes.yml").sql_verdicts;
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_row(
        &root,
        &rows,
        3,
        "relation:events:column:owner_id",
        SqlVerdictStatus::Abstain,
        Some("missing-relation-catalog"),
    );
}

#[test]
fn invalid_static_conflict_sql_obeys_fail_and_ignore_without_a_false_proof() {
    let root = fixture("conflict");
    let fail = report(&root, "fail.yml");
    let ignore = report(&root, "ignore.yml");
    let rule = "postgres-conflict-ordering";
    for rows in [&fail.sql_verdicts, &ignore.sql_verdicts] {
        let row = at_line(&root, rows, 4, rule, "canonical-conflict-order");
        assert_eq!(row.status, SqlVerdictStatus::Abstain, "{row:?}");
        assert!(
            row.residual_reasons
                .iter()
                .any(|reason| reason == "invalid-sql"),
            "{row:?}"
        );
    }
    let findings = fail
        .rules
        .iter()
        .filter(|finding| finding.rule == rule)
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 1, "{:?}", fail.rules);
    assert_eq!(findings[0].file, "src/query.ts");
    assert_eq!(findings[0].line, 4);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable-sql"));
    assert!(
        ignore.rules.iter().all(|finding| finding.rule != rule),
        "{:?}",
        ignore.rules
    );
}
