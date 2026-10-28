use super::{run_all_with_sql_verdicts, PathBuf, SqlVerdictStatus};

#[test]
fn conflict_ordering_keeps_complete_cap_and_loop_violations_and_unknown_residuals() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-conflict-ordering/variants/truncated-retained");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().to_path_buf();
    let path = root.join("src/query.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = no_mistakes::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &no_mistakes::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(embedded.calls.iter().all(|call| !call.variants.is_empty()));
    assert!(embedded.calls[3].variants[0]
        .sql_text
        .contains("deadlock-safe"));
    for policy in ["ignore", "report", "advise"] {
        let run = |include| {
            run_all_with_sql_verdicts(
                root.clone(),
                Some(root.join(format!("{policy}.yml"))),
                None,
                false,
                false,
                include,
            )
            .unwrap()
        };
        let plain = run(false);
        let detailed = run(true);
        assert_eq!(plain.rules, detailed.rules, "{policy}");
        assert_eq!(plain.advisories, detailed.advisories, "{policy}");
        assert_eq!(
            plain
                .rules
                .iter()
                .filter(|finding| finding.target.as_deref() == Some("missing-canonical-order"))
                .count(),
            3,
            "{policy}: {:?}",
            plain.rules
        );
        assert_eq!(
            plain
                .rules
                .iter()
                .filter(|finding| finding.target.as_deref() == Some("unanalyzable-sql"))
                .count(),
            if policy == "report" { 4 } else { 0 },
            "{policy}: {:?}",
            plain.rules
        );
        assert_eq!(
            plain.advisories.len(),
            if policy == "advise" { 4 } else { 0 },
            "{policy}: {:?}",
            plain.advisories
        );
        let obligations: Vec<_> = detailed
            .sql_verdicts
            .iter()
            .filter(|row| row.subcheck == "canonical-conflict-order")
            .collect();
        assert_eq!(obligations.len(), 4);
        assert_eq!(
            obligations
                .iter()
                .filter(|row| row.status == SqlVerdictStatus::Violation)
                .count(),
            3
        );
        assert_eq!(
            obligations
                .iter()
                .filter(|row| row.status == SqlVerdictStatus::Abstain)
                .count(),
            1
        );
        assert!(obligations.iter().all(|row| {
            row.residual_reasons
                .iter()
                .any(|reason| reason == "opaque-sql-tail")
        }));
    }
}
