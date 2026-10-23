use crate::check_runner::run_all_with_sql_verdicts;
use crate::codebase::postgres::SqlVerdictStatus;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn untrusted_tag_append_cannot_keep_a_stale_exact_clear_verdict() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/verdict-invalid-append",
    );
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().to_path_buf();
    let run = |include_sql_verdicts| {
        run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(".no-mistakes.yml")),
            None,
            false,
            false,
            include_sql_verdicts,
        )
        .unwrap()
    };
    let plain = run(false);
    let detailed = run(true);
    assert_eq!(plain.rules, detailed.rules);
    assert_eq!(plain.advisories, detailed.advisories);
    assert_eq!(plain.warnings, detailed.warnings);
    assert_eq!(
        plain
            .rules
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7]
    );
    let plain_json = crate::check_runner::json_value(&plain);
    assert!(plain_json.get("sqlVerdicts").is_none());
    assert_eq!(detailed.sql_verdicts.len(), 1);
    let verdict = &detailed.sql_verdicts[0];
    assert_eq!(verdict.file, "main.mts");
    assert_eq!(
        std::fs::read_to_string(root.join("main.mts")).unwrap()[..verdict.span.0 as usize]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1,
        7
    );
    assert_eq!(verdict.status, SqlVerdictStatus::Abstain, "{verdict:?}");
    assert_eq!(verdict.evidence, None);
    assert!(verdict.known_prefix.is_none());
}

#[test]
fn truncated_variant_ledger_uses_prepared_prefix_and_unknown_policy() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    use crate::codebase::postgres::{EmbeddedSqlKind, EmbeddedSqlOptions};
    use crate::codebase::rules::postgres_require_query_annotation::{Options, UnanalyzableSql};
    use crate::codebase::ts_resolver::normalize_path;

    let path = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/embedded/variants-truncated-verdict.ts"),
    );
    let root = path.parent().unwrap();
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    for (policy, advise) in [
        (UnanalyzableSql::Report, false),
        (UnanalyzableSql::Advise, true),
        (UnanalyzableSql::Ignore, false),
    ] {
        let mut facts = collect_check_facts(
            root,
            vec![path.clone()],
            CheckFactPlan {
                embedded_sql: true,
                embedded_sql_options: vec![profile.clone()],
                query_annotation: true,
                query_annotation_options: Some(vec![profile.clone()]),
                ..Default::default()
            },
        );
        let file = Arc::get_mut(facts.ts.get_mut(&path).unwrap())
            .expect("fixture facts have one request owner");
        let embedded = file.embedded_sql[0].1.clone();
        assert_eq!(embedded.calls.len(), 3);
        assert!(embedded.calls.iter().all(|call| call.variants_truncated()));
        let prefixes = [None, Some("/* fixed */ SELECT 2".to_string()), None];
        for ((call, span), prefix) in embedded
            .calls
            .iter()
            .zip(&embedded.call_spans)
            .zip(prefixes)
        {
            assert_eq!(call.kind, EmbeddedSqlKind::Composed);
            file.query_annotation[0].1.calls.insert(*span, prefix);
        }
        facts.enable_sql_verdicts();
        let options =
            crate::codebase::rules::postgres_require_query_annotation::compile_options(&Options {
                import_specifier: "@bench/db".into(),
                executor_names: vec!["read".into()],
                unanalyzable_sql: policy,
                ..Default::default()
            })
            .unwrap();
        super::super::record(
            "variants-truncated-verdict.ts",
            &embedded,
            &options,
            &facts,
            &Default::default(),
            &facts,
            &path,
        );
        let rows = facts.sql_verdicts();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].status, SqlVerdictStatus::Abstain);
        assert_eq!(rows[0].advisory, advise);
        assert_eq!(rows[0].evidence, None);
        assert!(rows[0]
            .residual_reasons
            .contains(&"sql-variants-truncated".into()));
        assert_eq!(rows[1].status, SqlVerdictStatus::ProvenClear);
        assert_eq!(
            rows[1].evidence,
            Some(crate::codebase::postgres::SqlEvidenceKind::VerifiedPrefix)
        );
        assert!(!rows[1].advisory);
        assert_eq!(rows[2].status, SqlVerdictStatus::Violation);
        assert_eq!(
            rows[2].evidence,
            Some(crate::codebase::postgres::SqlEvidenceKind::VerifiedPrefix)
        );
        assert!(!rows[2].advisory);
    }
}
