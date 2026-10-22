use super::{
    collect_check_fact_batch_with_session,
    collect_check_facts_with_graph_files_playwright_and_sources, BatchCheckFactRequest,
    CheckFactMap, CheckFactPlan, CheckFileFacts, PlaywrightFactPlan,
};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::config::v2::{load_v2_config, NoMistakesConfig};
use std::path::PathBuf;
use std::sync::Arc;

struct SavedInput {
    _fixture: tempfile::TempDir,
    root: PathBuf,
    files: Vec<PathBuf>,
    config: NoMistakesConfig,
    plan: CheckFactPlan,
    sources: Arc<SourceStore>,
}

impl SavedInput {
    fn new(scenario: &str) -> Self {
        let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
            .join(scenario);
        let fixture = crate::test_support::materialize_saved_fixture(&saved);
        let root = normalize_path(fixture.path());
        let files = discover_visible_paths(&root);
        let config = load_v2_config(&root, None).unwrap();
        let mut plan = CheckFactPlan {
            embedded_sql: true,
            embedded_sql_options:
                crate::codebase::postgres::configured_embedded_sql_options_for_checks(&config)
                    .unwrap(),
            ..Default::default()
        };
        crate::codebase::postgres::configure_prepared_postgres_plan(&config, &mut plan).unwrap();
        let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
            &files,
        ))));
        Self {
            _fixture: fixture,
            root,
            files,
            config,
            plan,
            sources,
        }
    }

    fn collect(&self, staged: bool) -> CheckFactMap {
        collect_check_facts_with_graph_files_playwright_and_sources(
            &self.root,
            self.files.clone(),
            Vec::new(),
            self.plan.clone(),
            staged.then(PlaywrightFactPlan::default),
            Arc::clone(&self.sources),
        )
    }

    fn request(&self, staged: bool) -> BatchCheckFactRequest {
        BatchCheckFactRequest {
            root: self.root.clone(),
            files: self.files.clone(),
            graph_files: Vec::new(),
            plan: self.plan.clone(),
            playwright: staged.then(PlaywrightFactPlan::default),
            sources: Arc::clone(&self.sources),
        }
    }
}

fn assert_program_retired(file: &CheckFileFacts) {
    assert!(!file.query_annotation.is_empty());
    for (_, annotation) in &file.query_annotation {
        let mut expected = QueryAnnotationFileFacts::default();
        expected.calls.clone_from(&annotation.calls);
        // Debug exposes the full lifecycle state without widening private
        // evaluator fields or reading protected implementation details.
        assert_eq!(format!("{annotation:?}"), format!("{expected:?}"));
    }
}

fn assert_reachable_report(input: &SavedInput, facts: &CheckFactMap) {
    let helper = input.root.join("src/helper.mts");
    let file = facts.ts.get(&helper).unwrap();
    assert_program_retired(file);
    assert_eq!(file.query_annotation.len(), 1);
    assert_eq!(
        file.query_annotation[0].0.executor_names,
        ["query", "read", "write"]
    );
    let profile = &file.query_annotation[0].0;
    let span = file
        .embedded_sql
        .iter()
        .find(|(options, _)| options == profile)
        .unwrap()
        .1
        .call_spans[0];
    assert_eq!(
        facts.query_annotation_prefix(&helper, profile, span),
        Some(&Some("SELECT 1".to_string()))
    );
    let findings = crate::codebase::rules::postgres_require_query_annotation::
        check_with_files_sources_and_facts(
            &input.root,
            &input.config,
            &input.files,
            &input.sources,
            facts,
        )
        .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "src/helper.mts");
    assert_eq!(findings[0].line, 3);
    assert_eq!(findings[0].target.as_deref(), Some("annotation"));
}

#[test]
fn ordinary_and_staged_annotation_fact_finalization_preserves_reports() {
    let input = SavedInput::new("helper-tracing-reachability");
    let raw = super::tests::collect_file_facts(
        &input.root,
        &input.root.join("src/helper.mts"),
        &input.plan,
        None,
    )
    .unwrap();
    assert_eq!(raw.query_annotation.len(), 1);
    assert_ne!(
        format!("{:?}", raw.query_annotation[0].1),
        format!("{:?}", QueryAnnotationFileFacts::default()),
        "the saved source must collect actual evaluator program state"
    );
    for staged in [false, true] {
        assert_reachable_report(&input, &input.collect(staged));
    }
}

#[test]
fn annotation_fact_finalization_in_shared_batch_preserves_each_projection() {
    let input = SavedInput::new("helper-tracing-reachability");
    crate::ast::begin_parse_count(&input.root);
    let maps = collect_check_fact_batch_with_session(
        &AnalysisSession::disabled(),
        vec![input.request(false), input.request(true)],
    );
    let counts = crate::ast::finish_parse_count(&input.root);
    assert_eq!(maps.len(), 2);
    for map in &maps {
        assert_reachable_report(&input, map);
    }
    for file in input
        .files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        assert_eq!(counts.get(file), Some(&1), "{counts:?}");
    }
}

#[test]
fn annotation_fact_finalization_retains_unknown_and_concrete_call_prefixes() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-shared-snapshots");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let profile = crate::codebase::postgres::EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = super::collect_check_facts(
        &root,
        discover_visible_paths(&root),
        CheckFactPlan {
            embedded_sql: true,
            query_annotation: true,
            embedded_sql_options: vec![profile.clone()],
            ..Default::default()
        },
    );
    let file = facts.ts.get(&root.join("src/query.mts")).unwrap();
    assert_program_retired(file);
    assert_eq!(file.query_annotation.len(), 1);
    assert_eq!(file.query_annotation[0].0, profile);
    let calls = &file.query_annotation[0].1.calls;
    assert_eq!(calls.len(), 3, "{calls:?}");
    assert!(calls.values().any(Option::is_none), "{calls:?}");
    assert!(
        calls
            .values()
            .any(|prefix| prefix.as_deref() == Some("SELECT 2")),
        "{calls:?}"
    );
}
