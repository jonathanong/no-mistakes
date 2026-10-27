use super::tests::{config, fixture};
use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SqlVerdictStatus;
use crate::codebase::ts_source::SourceStore;
use std::sync::Arc;

// Output cases moved to cli_postgres_snapshot_transformed_uuid_predicates.
// Keep the two-application cache, CRLF bytes, and verdict statuses below.

type Prepared = (
    NoMistakesConfig,
    Vec<PathBuf>,
    Arc<SourceStore>,
    CheckFactMap,
);
pub(super) fn prepare(name: &str, mut cfg: NoMistakesConfig) -> Prepared {
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
fn physical_crlf_templates_match_cooked_text_and_keep_physical_finding_lines() {
    let source = std::fs::read(fixture().join("crlf.ts")).unwrap();
    assert!(source.windows(2).any(|bytes| bytes == b"\r\n"));
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
        check_with_files_sources_and_facts(&fixture(), &cfg, &files, &sources, &facts).unwrap();
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
mod helper_recovery;
