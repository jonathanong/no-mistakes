use super::*;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    fixture: String,
    subcheck: String,
    known_span: (u32, u32),
    exact_span: (u32, u32),
    unknown_span: (u32, u32),
    unknown_line: usize,
}

fn run(root: &Path, mode: &str, include: bool) -> CheckResults {
    run_all_with_sql_verdicts(
        root.to_path_buf(),
        Some(root.join(format!("{mode}.yml"))),
        None,
        false,
        false,
        include,
    )
    .unwrap()
}

#[test]
fn every_sql_rule_keeps_proven_findings_across_modes_and_opt_in_reporting() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/mode-parity");
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(fixtures.join("cases.json")).unwrap())
            .unwrap();
    assert_eq!(cases.len(), 12);
    for case in cases {
        let root = fixtures.join(&case.fixture);
        let mut expected_findings = None;
        let mut expected_proofs = None;
        for mode in ["fail", "ignore", "advise"] {
            let plain = run(&root, mode, false);
            let evidence = run(&root, mode, true);
            // The optional inventory must not alter any existing report field.
            let plain_json = json_value(&plain);
            let mut evidence_json = json_value(&evidence);
            assert!(plain_json.get("sqlVerdicts").is_none());
            evidence_json.as_object_mut().unwrap().remove("sqlVerdicts");
            assert_eq!(
                serde_json::to_vec(&plain_json).unwrap(),
                serde_json::to_vec(&evidence_json).unwrap(),
                "{} {mode}: opt-in changed existing output",
                case.fixture
            );
            let proven = evidence
                .rules
                .iter()
                .filter(|finding| {
                    finding.file != "src/query.ts" || finding.line != case.unknown_line
                })
                .cloned()
                .collect::<Vec<_>>();
            assert!(
                !proven.is_empty(),
                "{} {mode}: lost proven finding",
                case.fixture
            );
            if let Some(expected) = &expected_findings {
                assert_eq!(&proven, expected, "{} {mode}", case.fixture);
            } else {
                expected_findings = Some(proven);
            }
            let known = evidence
                .sql_verdicts
                .iter()
                .find(|row| {
                    row.file == "src/query.ts"
                        && row.span == case.known_span
                        && row.subcheck == case.subcheck
                })
                .unwrap_or_else(|| panic!("{} {mode}: missing known call", case.fixture));
            assert_eq!(
                known.status,
                SqlVerdictStatus::Violation,
                "{} {mode}",
                case.fixture
            );
            assert_eq!(known.evidence, Some(SqlEvidenceKind::ExhaustiveUnion));
            let exact = evidence
                .sql_verdicts
                .iter()
                .find(|row| {
                    row.file == "src/query.ts"
                        && row.span == case.exact_span
                        && row.subcheck == case.subcheck
                })
                .unwrap_or_else(|| panic!("{} {mode}: missing exact call", case.fixture));
            assert_eq!(
                exact.status,
                SqlVerdictStatus::Violation,
                "{} {mode}",
                case.fixture
            );
            assert_eq!(exact.evidence, Some(SqlEvidenceKind::Exact));
            let unknown = evidence
                .sql_verdicts
                .iter()
                .filter(|row| row.file == "src/query.ts" && row.span == case.unknown_span)
                .collect::<Vec<_>>();
            assert!(
                !unknown.is_empty(),
                "{} {mode}: missing opaque call",
                case.fixture
            );
            assert!(unknown.iter().any(|row| row.subcheck == case.subcheck));
            for row in unknown {
                assert_eq!(row.status, SqlVerdictStatus::Abstain, "{row:?}");
                assert!(row.evidence.is_none(), "{row:?}");
                assert!(!row.residual_reasons.is_empty(), "{row:?}");
            }
            let uncertain_findings = evidence.rules.iter().filter(|finding| {
                finding.file == "src/query.ts" && finding.line == case.unknown_line
            });
            let uncertain_advice = evidence.advisories.iter().filter(|finding| {
                finding.file == "src/query.ts" && finding.line == case.unknown_line
            });
            assert_eq!(
                uncertain_findings.count() > 0,
                mode == "fail",
                "{} {mode}",
                case.fixture
            );
            assert_eq!(
                uncertain_advice.count() > 0,
                mode == "advise",
                "{} {mode}",
                case.fixture
            );
            let mut proofs = serde_json::to_value(&evidence.sql_verdicts).unwrap();
            for row in proofs.as_array_mut().unwrap() {
                row.as_object_mut().unwrap().remove("advisory");
            }
            if let Some(expected) = &expected_proofs {
                assert_eq!(
                    &proofs, expected,
                    "{} {mode}: proof changed with mode",
                    case.fixture
                );
            } else {
                expected_proofs = Some(proofs);
            }
        }
    }
}
