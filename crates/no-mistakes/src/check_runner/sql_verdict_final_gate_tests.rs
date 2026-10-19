use super::*;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(path)
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

#[test]
fn declared_partition_exemption_does_not_exempt_required_columns() {
    let base = "postgres-sql-verdicts/fixture/coverage-catalog";
    let catalog = fixture(&format!("{base}/catalog"));
    let exempt = report(&catalog, "exempt.yml");
    let source = std::fs::read_to_string(catalog.join("src/query.ts")).unwrap();
    let row = exempt
        .sql_verdicts
        .iter()
        .find(|row| {
            row.subcheck == "partition-key"
                && row.file == "src/query.ts"
                && source[..row.span.0 as usize]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    == 2
        })
        .unwrap();
    assert_eq!(row.status, SqlVerdictStatus::ProvenClear, "{row:?}");
    assert!(row.residual_reasons.is_empty(), "{row:?}");
    // The partition exemption does not exempt the independent owner-column check.
    let partition_span = row.span;
    let owner = exempt
        .sql_verdicts
        .iter()
        .find(|row| row.span == partition_span && row.subcheck == "relation:events:column:owner_id")
        .unwrap();
    assert_eq!(owner.status, SqlVerdictStatus::Violation, "{owner:?}");
}

#[test]
fn incomplete_star_projection_does_not_overclaim_a_verified_prefix() {
    let root = fixture("postgres-sql-verdicts/fixture/coverage-final/explicit-prefix");
    let result = report(&root, ".no-mistakes.yml");
    let row = result
        .sql_verdicts
        .iter()
        .find(|row| row.file == "src/query.ts" && row.subcheck == "select-star")
        .unwrap();
    assert_eq!(row.status, SqlVerdictStatus::Abstain, "{row:?}");
    assert_eq!(row.evidence, None);
    assert_eq!(row.residual_reasons, ["opaque-sql-tail"]);
}

#[test]
fn complete_offset_with_opaque_predicate_value_has_verified_prefix_evidence() {
    let root = fixture("postgres-sql-verdicts/fixture/coverage-final/no-offset-prefix");
    let result = report(&root, ".no-mistakes.yml");
    let row = result
        .sql_verdicts
        .iter()
        .find(|row| row.file == "src/query.ts" && row.subcheck == "offset")
        .unwrap();
    assert_eq!(row.status, SqlVerdictStatus::Violation, "{row:?}");
    assert_eq!(row.evidence, Some(SqlEvidenceKind::VerifiedPrefix));
    assert_eq!(row.residual_reasons, ["opaque-sql-tail"]);
}
