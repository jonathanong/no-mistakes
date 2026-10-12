use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};
use crate::codebase::dependencies::extract::{is_tsx_file, ImportExtractor};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

const FILES: &[&str] = &[
    "packages/app/package.json",
    "packages/app/index.ts",
    "packages/app/test-only.test.ts",
    "packages/lib/package.json",
    "packages/lib/index.mts",
    "packages/lib/priority.mts",
    "packages/lib/priority.ts",
    "packages/lib/folder/index.mts",
    "packages/lib/folder/index.ts",
    "packages/lib/rewritten.ts",
    "packages/lib/alias.ts",
    "packages/lib/extensionless",
    "packages/lib/plain.md",
    "packages/lib/broken.mts",
    "packages/lib/fatal.mts",
    "packages/lib/typed.js",
    "packages/lib/sub.mts",
    "packages/lib/type-only.ts",
    "packages/lib/computed.ts",
    "packages/lib/helper.test.mts",
    "packages/lib/through-test-helper.ts",
    "packages/test-lib/package.json",
    "packages/test-lib/index.ts",
    "tsconfig.json",
];

fn prepared(
    root: &Path,
    files: &[PathBuf],
    config: &NoMistakesConfig,
    sources: &Arc<SourceStore>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> crate::codebase::check_facts::CheckFactMap {
    collect_check_facts_with_graph_files_playwright_sources_and_session(
        session,
        root,
        (files.to_vec(), Vec::new()),
        CheckFactPlan {
            graph: import_fact_plan(),
            production_import_paths: required_import_fact_files(root, config, files).unwrap(),
            ..Default::default()
        },
        None,
        Arc::clone(sources),
    )
}

#[test]
fn prepared_import_projection_matches_legacy_syntax_for_every_fixture_path() {
    let root = fixture_root("prepared-projection-parity");
    let files = scenario_files(&root, FILES);
    let sources = super::super::super::source_store_for_files(&files);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = prepared(&root, &files, &config(""), &sources, &session);
    for file in &files {
        let source = sources.read_path(file).unwrap();
        let extractor = if is_tsx_file(file) {
            ImportExtractor::for_tsx()
        } else {
            ImportExtractor::for_typescript()
        }
        .unwrap();
        assert_eq!(
            facts.get_production_imports(file).unwrap(),
            extractor.extract(&source).unwrap(),
            "{}",
            file.display()
        );
    }
    // Recovered syntax is part of the old rule's contract, including imports
    // before a parser diagnostic and executable text in unknown extensions.
    for path in [
        "packages/lib/broken.mts",
        "packages/lib/plain.md",
        "packages/lib/extensionless",
    ] {
        assert!(!facts
            .get_production_imports(&root.join(path))
            .unwrap()
            .is_empty());
    }
    assert!(facts
        .get_production_imports(&root.join("packages/lib/fatal.mts"))
        .unwrap()
        .is_empty());
}

#[test]
fn prepared_rule_resolves_each_specifier_once_and_never_reads_or_parses_again() {
    let root = fixture_root("prepared-projection-parity");
    let files = scenario_files(&root, FILES);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = Arc::new(SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        Some(Arc::clone(&observer)),
    ));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let config = config("");
    let facts = prepared(&root, &files, &config, &sources, &session);
    let before = observer.snapshot().work;
    let parses = session.work_snapshot().parse_attempts;
    for (path, count) in [
        ("packages/app/index.ts", 1),
        ("packages/lib/index.mts", 2),
        ("packages/lib/typed.js", 2),
        ("packages/lib/plain.md", 1),
        ("packages/lib/extensionless", 1),
    ] {
        assert_eq!(parses.get(&root.join(path)), Some(&count), "{parses:#?}");
    }
    let reads = observer.source_read_snapshot();
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts)).unwrap();
    let after = observer.snapshot().work;
    assert_eq!(before.get("parse.files"), after.get("parse.files"));
    assert_eq!(reads, observer.source_read_snapshot());
    assert!(reads.values().all(|count| *count == 1));
    assert_eq!(after.get("production_dependencies.resolutions"), Some(&30));
    assert!(after
        .get("production_dependencies.legacy_import_projections")
        .is_some_and(|count| *count > 0));
    let mut targets: Vec<_> = findings
        .iter()
        .map(|finding| finding.target.as_deref().unwrap())
        .collect();
    targets.sort();
    assert_eq!(
        targets,
        vec![
            "@source/alias",
            "chosen-extension",
            "chosen-index",
            "extensionless-dependency",
            "javascript-dependency",
            "markdown-dependency",
            "recovered-dependency",
            "self-subpath-dependency",
            "through-test-helper-dependency"
        ]
    );
    let standalone = check_with_files(&root, &config, &files).unwrap();
    let dispatched =
        crate::codebase::rules::run_filesystem_rules_with_config(&root, &config, &files).unwrap();
    assert_eq!(
        serde_json::to_vec(&findings).unwrap(),
        serde_json::to_vec(&standalone).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&findings).unwrap(),
        serde_json::to_vec(&dispatched).unwrap()
    );
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let boundary = crate::codebase::rules::run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
        &root, &config, &files, crate::codebase::rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
            snapshot: &snapshot, vitest_catalog: None, sources: snapshot.source_store_for(&root),
            workflow_documents: None, tsconfig_gate_project_inputs: None, config_path: None,
        }, None,
    ).unwrap();
    assert_eq!(
        serde_json::to_vec(&findings).unwrap(),
        serde_json::to_vec(&boundary).unwrap()
    );
}

#[test]
fn prepared_rule_keeps_workspace_root_scope_and_failure_entries() {
    let root = fixture_root("prepared-projection-parity");
    let mut files = scenario_files(&root, FILES);
    files.push(root.join("packages/lib/unreadable.mts"));
    let sources = super::super::super::source_store_for_files(&files);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let scoped = config_with_roots(&["packages/lib"]);
    let facts = prepared(&root, &files, &scoped, &sources, &session);
    assert!(facts
        .get_production_imports(&root.join("packages/lib/unreadable.mts"))
        .unwrap()
        .is_empty());
    assert!(!facts
        .get_ts_facts(&root.join("packages/lib/unreadable.mts"))
        .unwrap()
        .operational_error
        .as_ref()
        .unwrap()
        .is_empty());
    assert!(
        check_with_files_sources_and_facts(&root, &scoped, &files, &sources, Some(&facts))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn prepared_rule_rejects_incomplete_fact_plans_instead_of_reparsing() {
    let root = fixture_root("prepared-projection-parity");
    let files = scenario_files(&root, FILES);
    let sources = super::super::super::source_store_for_files(&files);
    let sparse = crate::codebase::ts_source::facts::TsFactMap::default();
    let error =
        check_with_files_sources_and_facts(&root, &config(""), &files, &sources, Some(&sparse))
            .unwrap_err();
    assert!(error.to_string().contains("complete prepared import plan"));
    let imports_only =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            CheckFactPlan {
                graph: import_fact_plan(),
                ..Default::default()
            },
            None,
            Arc::clone(&sources),
        );
    let error = check_with_files_sources_and_facts(
        &root,
        &config(""),
        &files,
        &sources,
        Some(&imports_only),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("missing prepared production import facts"));
}

#[test]
fn production_imports_preserve_mdx_storybook_raw_source_and_legacy_symbol_demand() {
    let root = fixture_root("prepared-projection-parity");
    let story = root.join("packages/lib/story.mdx");
    let metadata = root.join("packages/lib/plain.md");
    let files = vec![story.clone(), metadata.clone()];
    let sources = super::super::super::source_store_for_files(&files);
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            CheckFactPlan {
                graph: import_fact_plan(),
                storybook: true,
                raw_source: true,
                symbols: true,
                legacy_symbol_paths: [metadata.clone()].into_iter().collect(),
                production_import_paths: Arc::new(files.into_iter().collect()),
                ..Default::default()
            },
            None,
            Arc::clone(&sources),
        );
    let mdx = facts.ts.get(&story).unwrap();
    assert_eq!(
        mdx.source.as_deref(),
        Some(sources.read_path(&story).unwrap().as_ref())
    );
    let storybook = mdx.storybook.as_ref().unwrap();
    assert_eq!(storybook.used_runtime_imports.len(), 1);
    assert_eq!(storybook.used_runtime_imports[0].imported, "Meta");
    // MDX retains its Storybook facts, while the historical TypeScript import
    // parser rejects JSX and returns no production imports for this source.
    let legacy_imports = ImportExtractor::for_typescript()
        .unwrap()
        .extract(&sources.read_path(&story).unwrap())
        .unwrap();
    assert!(legacy_imports.is_empty());
    assert_eq!(
        facts.get_production_imports(&story).unwrap(),
        legacy_imports
    );
    let other = facts.ts.get(&metadata).unwrap();
    assert!(other.source.is_some());
    assert!(other.legacy_symbols.is_some());
    assert!(other.legacy_symbol_parse_error.is_none());
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn production_imports_complete_prepared_runner_helpers_before_fact_merge() {
    let saved = fixture_root("prepared-projection-parity");
    // Physical parser counting must use a private root so parallel parity tests
    // on the saved fixture cannot enter this test's accounting scope.
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let mut files = scenario_files(&root, FILES);
    files.extend(scenario_files(
        &root,
        &[
            "packages/app/vitest.config.mts",
            "packages/app/runner-helper.mts",
            "packages/app/typed-runner.js",
        ],
    ));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = Arc::new(SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        Some(Arc::clone(&observer)),
    ));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".runner-no-mistakes.yml")))
            .unwrap();
    let catalog = Arc::new(
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            &root,
            &[],
            &files,
            &sources,
        ),
    );
    let runner = Arc::new(
        crate::integration_tests::prepare_runner_configs_with_catalog(
            &root,
            &config,
            &files,
            catalog,
            Arc::clone(&sources),
        ),
    );
    crate::ast::begin_parse_count(&root);
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (files.clone(), Vec::new()),
        CheckFactPlan {
            graph: import_fact_plan(),
            production_import_paths: required_import_fact_files(&root, &config, &files).unwrap(),
            integration_runner_configs: Some(runner),
            ..Default::default()
        },
        None,
        Arc::clone(&sources),
    );
    let counts = crate::ast::finish_parse_count(&root);
    for (name, dependency) in [
        ("runner-helper.mts", "runner-mts-dependency"),
        ("typed-runner.js", "runner-js-dependency"),
    ] {
        let file = root.join("packages/app").join(name);
        assert_eq!(counts.get(&file), Some(&2), "{counts:#?}");
        assert!(facts
            .get_production_imports(&file)
            .unwrap()
            .iter()
            .any(|import| import.specifier == dependency));
    }
    assert!(observer
        .source_read_snapshot()
        .values()
        .all(|count| *count == 1));
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts)).unwrap();
    assert_eq!(findings.len(), 9);
}
