use super::*;
use no_mistakes::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-replay")
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
    line: usize,
    subcheck: &str,
) -> &'a SqlObligationVerdict {
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let matches = rows
        .iter()
        .filter(|row| {
            row.file == "src/query.ts"
                && row.rule == "postgres-idempotent-insert"
                && row.subcheck == subcheck
                && source[..row.span.0 as usize]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1
                    == line
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "src/query.ts:{line} {subcheck}");
    matches[0]
}

#[test]
fn guarded_insert_keeps_trigger_judgment_independent_of_replay_guard() {
    let root = fixture();
    let rows = report(&root).sql_verdicts;
    for line in [4, 5] {
        let guard = at_line(&root, &rows, line, "replay-guard");
        assert_eq!(guard.status, SqlVerdictStatus::ProvenClear, "{guard:?}");
    }
    let clear = at_line(&root, &rows, 4, "trigger-replay");
    assert_eq!(clear.status, SqlVerdictStatus::ProvenClear, "{clear:?}");
    let unsafe_trigger = at_line(&root, &rows, 5, "trigger-replay");
    assert_eq!(
        unsafe_trigger.status,
        SqlVerdictStatus::Violation,
        "{unsafe_trigger:?}"
    );
}

#[test]
fn unsupported_arbiter_abstains_while_known_noop_conflict_clears() {
    let root = fixture();
    let rows = report(&root).sql_verdicts;
    let uncertain = at_line(&root, &rows, 7, "arbiter");
    assert_eq!(uncertain.status, SqlVerdictStatus::Abstain, "{uncertain:?}");
    assert_eq!(uncertain.residual_reasons, ["unresolved-replay-safety"]);
    let clear = at_line(&root, &rows, 9, "trigger-replay");
    assert_eq!(clear.status, SqlVerdictStatus::ProvenClear, "{clear:?}");
}

#[test]
fn invalid_sql_known_prefix_and_opaque_text_have_distinct_residuals() {
    let root = fixture();
    let rows = report(&root).sql_verdicts;
    let invalid = at_line(&root, &rows, 11, "replay-guard");
    assert_eq!(invalid.status, SqlVerdictStatus::Abstain, "{invalid:?}");
    assert!(
        invalid
            .residual_reasons
            .contains(&"invalid-sql".to_string()),
        "{invalid:?}"
    );
    let prefix = at_line(&root, &rows, 12, "replay-guard");
    assert_eq!(prefix.status, SqlVerdictStatus::Abstain, "{prefix:?}");
    assert!(prefix
        .known_prefix
        .as_deref()
        .is_some_and(|sql| sql.starts_with("INSERT INTO items (")));
    assert_eq!(prefix.residual_reasons, ["opaque-sql-tail"]);
    let opaque = at_line(&root, &rows, 13, "replay-guard");
    assert_eq!(opaque.status, SqlVerdictStatus::Abstain, "{opaque:?}");
    assert!(
        opaque
            .residual_reasons
            .contains(&"unresolved-sql-fragment".to_string()),
        "{opaque:?}"
    );
    assert!(
        opaque
            .residual_reasons
            .contains(&"opaque-sql-tail".to_string()),
        "{opaque:?}"
    );
}
