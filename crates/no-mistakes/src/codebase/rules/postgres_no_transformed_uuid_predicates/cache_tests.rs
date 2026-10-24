use super::tests::{config, fixture};
use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SqlVerdictStatus;
use crate::codebase::ts_source::SourceStore;
use std::sync::Arc;

type Prepared = (
    NoMistakesConfig,
    Vec<PathBuf>,
    Arc<SourceStore>,
    CheckFactMap,
);
fn prepare(name: &str, mut cfg: NoMistakesConfig) -> Prepared {
    let root = fixture();
    let files = vec![root.join(name), root.join("schema.json")];
    let sources = super::super::source_store_for_files(&files);
    super::super::analysis_policy::prepare_rule_analysis(&root, &mut cfg, &files, &sources)
        .unwrap();
    let mut plan = crate::codebase::check_facts::CheckFactPlan {
        postgres_schema_catalog_paths: vec!["schema.json".into()],
        ..Default::default()
    };
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &cfg, &mut plan).unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&cfg, &mut plan).unwrap();
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            plan,
            None,
            sources.clone(),
        );
    (cfg, files, sources, facts)
}
#[test]
fn mixed_max_application_initializes_low_cache_without_parsing_literals_again() {
    let mut cfg = config("");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let mut max = cfg.rules[0].clone();
    max.options
        .as_mapping_mut()
        .unwrap()
        .insert("effort".into(), "max".into());
    cfg.rules.push(max);
    let (cfg, files, sources, facts) = prepare("embedded.ts", cfg);
    assert!(facts.ts.values().any(|file| !file.embedded_sql.is_empty()));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let findings = crate::diagnostics::with_observer(Some(observer.clone()), || {
        check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap()
    });
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert_eq!(
        facts.ts[&fixture().join("embedded.ts")]
            .sql_identifier_syntax
            .transformed_literals
            .get()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.no_transformed_uuid_predicates.low.literal_parses")
            .copied()
            .unwrap_or(0),
        0
    );
}
#[test]
fn escapes_interpolations_and_builder_calls_abstain_at_low_effort() {
    let mut cfg = config("unanalyzableSql: advise\n");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let (cfg, files, sources, mut facts) = prepare("low-inputs.ts", cfg);
    facts.enable_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 4, "{verdicts:#?}");
    assert_eq!(
        verdicts
            .iter()
            .filter(|verdict| verdict.status == SqlVerdictStatus::Abstain && verdict.advisory)
            .count(),
        3
    );
    assert_eq!(
        verdicts
            .iter()
            .filter(|verdict| verdict.status == SqlVerdictStatus::ProvenClear)
            .count(),
        1
    );
    let mut fail = cfg.clone();
    fail.rules[0].resolved_analysis = None;
    fail.rules[0]
        .options
        .as_mapping_mut()
        .unwrap()
        .insert("unanalyzableSql".into(), "fail".into());
    let findings = check_with_files(&fixture(), &fail, &files).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 4, 6],
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("unanalyzable")));
}
#[test]
fn physical_crlf_templates_match_cooked_text_and_keep_physical_finding_lines() {
    let root = fixture();
    let files = [root.join("crlf.ts"), root.join("schema.json")];
    let source = std::fs::read(root.join("crlf.ts")).unwrap();
    assert!(source.windows(2).any(|bytes| bytes == b"\r\n"));
    let max = check_with_files(&root, &config(""), &files).unwrap();
    let mut cfg = config("");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let low = check_with_files(&root, &cfg, &files).unwrap();
    assert_eq!(low, max);
    assert_eq!(
        low.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        [4, 6]
    );
}

#[test]
fn verdicts_distinguish_non_uuid_unknown_and_definite_transformed_columns() {
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        let mut cfg = config("unanalyzableSql: advise\n");
        cfg.effort = Some(effort);
        let (cfg, files, sources, mut facts) = prepare("verdict-inputs.ts", cfg);
        facts.enable_sql_verdicts();
        let findings =
            check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
        assert_eq!(findings.len(), 2, "{findings:#?}");
        let verdicts = facts.sql_verdicts();
        assert_eq!(
            verdicts
                .iter()
                .map(|verdict| verdict.status)
                .collect::<Vec<_>>(),
            [
                SqlVerdictStatus::ProvenClear,
                SqlVerdictStatus::Abstain,
                SqlVerdictStatus::Abstain,
                SqlVerdictStatus::Violation,
                SqlVerdictStatus::Violation
            ],
            "{verdicts:#?}"
        );
    }
}
#[test]
fn low_owner_uncertainty_produces_advice_without_enabling_all_verdicts() {
    let mut cfg = config("unanalyzableSql: advise\n");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let (cfg, files, sources, facts) = prepare("verdict-inputs.ts", cfg);
    assert!(!facts.wants_sql_verdicts());
    check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
    let records = facts.sql_verdicts();
    assert_eq!(records.len(), 2, "{records:#?}");
    assert!(records
        .iter()
        .all(|record| record.advisory && record.status == SqlVerdictStatus::Abstain));
}
#[test]
fn recovered_parameters_inside_casts_and_functions_are_not_columns() {
    let root = fixture();
    let files = [
        root.join("parameter-functions.ts"),
        root.join("schema.json"),
    ];
    let cfg = config("trustedSqlTags: [{module: db, name: sql}]\n");
    assert!(check_with_files(&root, &cfg, &files).unwrap().is_empty());
}
#[test]
fn values_and_parenthesized_query_bodies_still_visit_inner_predicates() {
    let root = fixture();
    let files = [root.join("values.sql"), root.join("schema.json")];
    let findings = check_with_files(&root, &config(""), &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 2);
}
