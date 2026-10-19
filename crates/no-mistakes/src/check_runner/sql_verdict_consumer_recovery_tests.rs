use super::*;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
use std::path::PathBuf;

#[test]
fn targetless_enum_insert_is_a_proven_finding_in_every_sql_mode() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-conflict-recovery");
    for mode in ["fail", "ignore", "advise"] {
        let run = |include| {
            run_all_with_sql_verdicts(
                root.clone(),
                Some(root.join(format!("{mode}.yml"))),
                None,
                false,
                false,
                include,
            )
            .unwrap()
        };
        let plain = run(false);
        let detailed = run(true);
        assert_eq!(plain.rules, detailed.rules, "{mode}");
        assert_eq!(plain.advisories, detailed.advisories, "{mode}");
        assert!(plain.advisories.is_empty(), "{mode}");
        assert_eq!(plain.rules.len(), 1, "{mode}: {:?}", plain.rules);
        let finding = &plain.rules[0];
        assert_eq!(finding.rule, "postgres-conflict-ordering", "{mode}");
        assert_eq!(finding.file, "src/query.ts", "{mode}");
        assert_eq!(finding.line, 4, "{mode}");
        assert_eq!(
            finding.target.as_deref(),
            Some("targetless-arbiter"),
            "{mode}"
        );
        let mut with_ledger = json_value(&detailed);
        assert_eq!(detailed.sql_verdicts.len(), 1, "{mode}");
        let verdict = &detailed.sql_verdicts[0];
        assert_eq!(verdict.file, "src/query.ts", "{mode}");
        assert_eq!(verdict.subcheck, "canonical-conflict-order", "{mode}");
        assert_eq!(verdict.status, SqlVerdictStatus::Violation, "{mode}");
        assert_eq!(verdict.evidence, Some(SqlEvidenceKind::Exact), "{mode}");
        assert!(verdict.residual_reasons.is_empty(), "{mode}");
        with_ledger.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(with_ledger, json_value(&plain), "{mode}");
    }
}
