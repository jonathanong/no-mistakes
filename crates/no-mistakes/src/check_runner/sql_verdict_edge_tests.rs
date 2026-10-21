use super::*;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlObligationVerdict, SqlVerdictStatus};
use std::path::Path;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/edge-cases")
        .join(name)
}

fn report(root: &Path) -> CheckResults {
    run_all_with_sql_verdicts(
        root.to_path_buf(),
        Some(root.join(".no-mistakes.yml")),
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
    file: &str,
    line: usize,
    rule: &str,
    subcheck: &str,
) -> &'a SqlObligationVerdict {
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

#[test]
fn finite_helper_alternatives_do_not_make_an_over_cap_call_exhaustive() {
    let root = fixture("cap");
    let rows = report(&root).sql_verdicts;
    let rule = "postgres-required-predicates";
    let check = "relation:view_active_plans:predicate:tier =";
    let file = "src/caller.mts";
    let finite = at_line(&root, &rows, file, 16, rule, check);
    assert_eq!(finite.status, SqlVerdictStatus::ProvenClear);
    assert_eq!(finite.evidence, Some(SqlEvidenceKind::ExhaustiveUnion));
    let capped = at_line(&root, &rows, file, 27, rule, check);
    assert_eq!(capped.status, SqlVerdictStatus::Abstain);
    assert_eq!(capped.evidence, None);
    assert!(capped
        .residual_reasons
        .iter()
        .any(|reason| reason == "opaque-sql-tail"));
    let composed = at_line(&root, &rows, file, 37, rule, check);
    assert_eq!(composed.status, SqlVerdictStatus::ProvenClear);
    assert_eq!(composed.evidence, Some(SqlEvidenceKind::ExhaustiveUnion));
}

#[test]
fn appended_limit_and_lock_are_judged_at_the_executor_call() {
    let root = fixture("appended");
    let rows = report(&root).sql_verdicts;
    let file = "src/query.ts";
    let bounded = "postgres-bounded-statements";
    let lock = "postgres-lock-ordering";
    assert_eq!(
        at_line(&root, &rows, file, 7, bounded, "select").status,
        SqlVerdictStatus::ProvenClear
    );
    assert_eq!(
        at_line(&root, &rows, file, 10, bounded, "select").status,
        SqlVerdictStatus::Violation
    );
    assert_eq!(
        at_line(&root, &rows, file, 15, lock, "row-lock-order").status,
        SqlVerdictStatus::Violation
    );
    assert_eq!(
        at_line(&root, &rows, file, 19, lock, "row-lock-order").status,
        SqlVerdictStatus::ProvenClear
    );
}

#[test]
fn a_known_relation_joined_to_an_unknown_peer_keeps_owner_evidence_open() {
    let root = fixture("..").join("unknown-owner");
    let rows = report(&root).sql_verdicts;
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = at_line(
        &root,
        &rows,
        "src/query.ts",
        4,
        "postgres-required-predicates",
        "relation:orders:column:tenant_id",
    );
    assert_eq!(row.status, SqlVerdictStatus::Abstain);
    assert_eq!(row.evidence, None);
    assert_eq!(row.residual_reasons, ["unknown-relation-owner".to_string()]);
}
