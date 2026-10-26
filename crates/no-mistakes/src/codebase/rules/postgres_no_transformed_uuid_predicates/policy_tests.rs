use super::tests::{config, fixture};
use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SqlVerdictStatus;
fn run(
    name: &str,
    effort: crate::config::v2::Effort,
    policy: &str,
) -> (Vec<RuleFinding>, CheckFactMap) {
    let root = fixture();
    let mut cfg = config(&format!("unanalyzableSql: {policy}\n"));
    cfg.effort = Some(effort);
    let files = vec![root.join(name), root.join("schema.json")];
    let sources = super::super::source_store_for_files(&files);
    super::super::analysis_policy::prepare_rule_analysis(&root, &mut cfg, &files, &sources)
        .unwrap();
    let mut plan =
        crate::codebase::check_facts::CheckFactPlan {
            postgres_schema_catalog_paths:
                crate::codebase::postgres::configured_schema_catalog_paths(&cfg, &[RULE_ID])
                    .unwrap(),
            ..Default::default()
        };
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &cfg, &mut plan).unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&cfg, &mut plan).unwrap();
    let mut facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            plan,
            None,
            sources.clone(),
        );
    facts.enable_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&root, &cfg, &files, &sources, &facts).unwrap();
    (findings, facts)
}
#[test]
fn raw_sql_parse_failures_and_ambiguous_owners_follow_all_policies_at_both_efforts() {
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        for (name, reason) in [
            ("invalid.sql", "invalid-sql"),
            ("ambiguous.sql", "ambiguous-column-owner"),
        ] {
            let (failures, _) = run(name, effort, "fail");
            assert_eq!(failures.len(), 1, "{failures:#?}");
            let (failures, facts) = run(name, effort, "ignore");
            assert!(failures.is_empty());
            assert!(facts.sql_verdicts().is_empty());
            let (failures, facts) = run(name, effort, "advise");
            assert!(failures.is_empty());
            let records = facts.sql_verdicts();
            assert_eq!(records.len(), 1, "{records:#?}");
            assert!(records[0].advisory && records[0].status == SqlVerdictStatus::Abstain);
            assert!(records[0].span.1 > records[0].span.0);
            assert_eq!(records[0].residual_reasons, [reason]);
            assert!(records[0].application.is_some());
        }
    }
}
#[test]
fn invalid_embedded_literals_are_never_proven_clear_at_either_effort() {
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        let (findings, _) = run("invalid.ts", effort, "fail");
        assert!(!findings.is_empty());
        let (findings, facts) = run("invalid.ts", effort, "advise");
        assert!(findings.is_empty());
        let verdicts = facts.sql_verdicts();
        assert_eq!(verdicts.len(), 1, "{verdicts:#?}");
        assert_eq!(verdicts[0].status, SqlVerdictStatus::Abstain);
        assert!(verdicts[0].advisory);
    }
}
