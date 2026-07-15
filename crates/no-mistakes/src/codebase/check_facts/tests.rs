use super::{
    collect_check_facts, collect_check_facts_with_graph_files_and_playwright,
    collect_check_facts_with_playwright, CheckFactMap, CheckFactPlan, PlaywrightFactPlan,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) fn collect_file_facts(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> Option<super::CheckFileFacts> {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(&[
        path.to_path_buf(),
    ]));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    super::collect_file_facts_with_session_and_sources(
        &session, root, path, plan, playwright, &sources,
    )
}

#[path = "tests/patch_coverage.rs"]
mod patch_coverage;

impl CheckFactMap {
    pub(crate) fn graph_file_universe_is_complete(&self) -> bool {
        self.graph_files_complete
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/shared-facts/fixture")
        .join(name)
}

fn ast_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-source/fixture/facts")
        .join(name)
}

fn playwright_plan(path: PathBuf) -> PlaywrightFactPlan {
    let mut plan = PlaywrightFactPlan::default();
    plan.add_file(super::PlaywrightFactSelection {
        path,
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_attributes: &["data-testid".to_string()],
        component_selector_attributes: &BTreeMap::new(),
        html_ids: false,
        test_id_attributes: &["data-testid".to_string()],
        policy: crate::playwright::playwright_tests::TestPolicy::default(),
        demands_text_imports: true,
    });
    plan
}

#[test]
fn collect_check_facts_records_read_errors() {
    let root = fixture_path("");
    let file = fixture_path("src/unreadable.ts");
    let facts = collect_check_facts(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            source: true,
            ..CheckFactPlan::default()
        },
    );

    assert_eq!(facts.stats.parse_errors, 1);
    let file_facts = facts.ts.get(&file).expect("read error fact is recorded");
    assert!(file_facts
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
}

#[test]
fn collect_check_facts_skips_non_indexable_files_with_minimal_plan() {
    let root = fixture_path("");
    let file = fixture_path("src/everything.tsx");
    let non_indexable = fixture_path("README.md");
    let facts = collect_check_facts(
        &root,
        vec![file.clone(), non_indexable],
        CheckFactPlan::default(),
    );

    assert_eq!(facts.stats.files_discovered, 2);
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(facts.stats.parse_errors, 0);
    assert_eq!(facts.graph_file_universe(), facts.files());
    assert_eq!(facts.ts.len(), 1);
    let file_facts = facts.ts.get(&file).expect("indexable file is parsed");
    assert!(file_facts.ts.imports.is_empty());
    assert!(file_facts.symbols.is_none());
    assert!(file_facts.source.is_none());
}

#[test]
fn collect_check_facts_records_parse_error_details() {
    let root = fixture_path("");
    let file = fixture_path("src/invalid.ts");
    let facts = collect_check_facts(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            source: true,
            ..CheckFactPlan::default()
        },
    );
    let parse_error = facts
        .ts
        .get(&file)
        .and_then(|facts| facts.parse_error.as_ref())
        .expect("parse error is recorded");

    assert_eq!(facts.stats.parse_errors, 1);
    assert_ne!(parse_error, &file.display().to_string());
    assert!(!parse_error.is_empty());
    let file_facts = facts.ts.get(&file).expect("file facts are retained");
    assert!(file_facts.source.is_some());
    assert!(file_facts.ts.imports.is_empty());
    assert!(file_facts.symbols.is_none());
}

#[test]
fn collect_check_facts_imports_include_reachability_metadata() {
    let root = fixture_path("");
    let file = fixture_path("src/everything.tsx");
    let facts = collect_check_facts(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            imports: true,
            ..CheckFactPlan::default()
        },
    );
    let file_facts = facts.ts.get(&file).expect("file facts are retained");

    assert!(file_facts
        .ts
        .imports
        .iter()
        .any(|import| import.specifier == "./widget"));
    assert!(
        file_facts
            .ts
            .function_calls
            .iter()
            .any(|call| call.callee == "Widget"),
        "shared import facts should include call metadata for graph reachability"
    );
}

#[test]
fn collect_check_facts_reads_raw_source_without_parsing() {
    let root = fixture_path("");
    let file = fixture_path("src/invalid.ts");
    let facts = collect_check_facts(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            raw_source: true,
            ..CheckFactPlan::default()
        },
    );
    let file_facts = facts.ts.get(&file).expect("file facts are retained");

    assert_eq!(facts.stats.files_parsed, 0);
    assert_eq!(facts.stats.parse_errors, 0);
    assert!(file_facts.source.is_some());
    assert!(file_facts.ts.source.is_some());
    assert_eq!(
        file_facts.source.as_deref(),
        file_facts.ts.source.as_deref(),
    );
    assert!(file_facts.parse_error.is_none());
}

#[test]
fn collect_check_facts_keeps_raw_source_when_parsing_is_required() {
    let root = fixture_path("");
    let file = fixture_path("src/everything.tsx");
    let facts = collect_check_facts(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            react: true,
            raw_source: true,
            ..CheckFactPlan::default()
        },
    );
    let file_facts = facts.ts.get(&file).expect("file facts are retained");

    assert_eq!(facts.stats.files_parsed, 1);
    assert!(file_facts.react.is_some());
    assert!(file_facts.source.is_some());
}

#[test]
fn collect_file_facts_keeps_raw_source_for_parse_and_source_type_errors() {
    let root = ast_fixture_path("");
    let unsupported = ast_fixture_path("unknown-extension.source");
    let facts = collect_file_facts(
        &root,
        &unsupported,
        &CheckFactPlan {
            react: true,
            raw_source: true,
            source: false,
            ..CheckFactPlan::default()
        },
        None,
    )
    .expect("unsupported source type fact is recorded");
    assert!(facts.source.is_some());
    assert!(facts.ts.source.is_some());
    assert!(facts.parse_error.is_some());

    let root = fixture_path("");
    let invalid = fixture_path("src/invalid.ts");
    let facts = collect_file_facts(
        &root,
        &invalid,
        &CheckFactPlan {
            react: true,
            raw_source: true,
            source: false,
            ..CheckFactPlan::default()
        },
        None,
    )
    .expect("parse error fact is recorded");
    assert!(facts.source.is_some());
    assert!(facts.ts.source.is_some());
    assert!(facts.parse_error.is_some());
}

#[test]
fn collect_file_facts_records_unsupported_source_type() {
    let root = ast_fixture_path("");
    let file = ast_fixture_path("unknown-extension.source");
    let facts = collect_file_facts(
        &root,
        &file,
        &CheckFactPlan {
            source: true,
            ..CheckFactPlan::default()
        },
        None,
    )
    .expect("unsupported source type fact is recorded");

    assert!(facts.source.is_some());
    assert!(facts
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("unsupported file type")));
}

#[test]
fn collect_check_facts_parses_once_for_overlapping_fact_categories() {
    let root = fixture_path("");
    let file = fixture_path("src/everything.tsx");
    let facts = collect_check_facts(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            imports: true,
            symbols: true,
            react: true,
            queue: true,
            queue_factory_names: vec![],
            integration: true,
            dynamic_imports: true,
            nextjs_caching: true,
            storybook: true,
            source: true,
            raw_source: false,
            ..CheckFactPlan::default()
        },
    );

    assert_eq!(facts.stats.files_discovered, 1);
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(facts.stats.parse_errors, 0);
    assert!(facts
        .graph_plan()
        .covers(crate::codebase::ts_source::facts::TsFactPlan {
            imports: true,
            function_calls: true,
            symbols: true,
            source: true,
            queue_project: true,
            react: true,
            ..Default::default()
        }));
    let file_facts = facts.ts.get(&file).expect("file facts are collected");
    let prepared_rule_view = std::sync::Arc::clone(file_facts);
    let prepared_graph_view = facts.ts.get(&file).cloned().unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &prepared_rule_view,
        &prepared_graph_view,
    ));
    assert!(!file_facts.ts.imports.is_empty());
    assert!(file_facts.symbols.is_some());
    assert!(file_facts.react.is_some());
    assert_eq!(
        file_facts.symbols.as_deref(),
        file_facts.ts.symbols.as_ref(),
    );
    assert_eq!(
        format!("{:?}", file_facts.react.as_ref().unwrap().components),
        format!("{:?}", file_facts.ts.react_components),
    );
    assert!(file_facts.ts.queue_project.is_some());
    assert!(file_facts.integration.is_some());
    assert!(file_facts.nextjs_caching.is_some());
    assert!(file_facts.dynamic_imports.is_some());
    assert!(file_facts.source.is_some());
}

#[test]
fn mdx_records_empty_boundary_facts_for_combined_storybook_plan() {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/require-storybook-stories/fixture/mdx-story/stories/card.mdx",
    );
    let root = file.ancestors().nth(2).unwrap();
    let facts = collect_file_facts(
        root,
        &file,
        &CheckFactPlan {
            storybook: true,
            server_route_client_boundary: true,
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert!(facts.storybook.is_some());
    assert_eq!(
        facts.server_route_client_boundary,
        Some(crate::codebase::rules::server_route_client_boundary::FileFacts::default())
    );
}

#[test]
fn collect_check_facts_parses_once_for_playwright_and_shared_facts() {
    let root = fixture_path("");
    let file = fixture_path("src/everything.tsx");
    let facts = collect_check_facts_with_playwright(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            react: true,
            ..CheckFactPlan::default()
        },
        Some(playwright_plan(file.clone())),
    );

    assert_eq!(facts.stats.files_discovered, 1);
    assert_eq!(facts.stats.files_parsed, 1);
    let file_facts = facts.ts.get(&file).expect("file facts are collected");
    assert!(file_facts.react.is_some());
    assert!(file_facts.playwright.is_some());
}

#[test]
fn collect_check_facts_keeps_graph_files_out_of_shared_file_scope() {
    let root = fixture_path("");
    let scoped = fixture_path("src/everything.tsx");
    let graph_only = fixture_path("src/widget.tsx");
    let facts = collect_check_facts_with_graph_files_and_playwright(
        &root,
        vec![scoped.clone()],
        vec![graph_only.clone()],
        CheckFactPlan {
            imports: true,
            queue: true,
            integration: true,
            graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
            ..CheckFactPlan::default()
        },
        None,
    );

    assert_eq!(facts.files(), std::slice::from_ref(&scoped));
    assert_eq!(
        facts.graph_file_universe(),
        std::slice::from_ref(&graph_only)
    );
    assert_eq!(
        crate::codebase::dependencies::graph::TsFactLookup::graph_files(&facts),
        Some(std::slice::from_ref(&graph_only))
    );
    assert!(facts.ts.contains_key(&scoped));
    assert!(facts.ts.contains_key(&graph_only));
    let graph_only_facts = facts.ts.get(&graph_only).expect("graph-only facts");
    assert!(!graph_only_facts.ts.imports.is_empty());
    assert!(graph_only_facts.ts.queue_project.is_none());
    assert!(graph_only_facts.integration.is_none());
    assert!(facts
        .graph_plan()
        .covers(crate::codebase::ts_source::facts::TsFactPlan::imports()));
    assert!(!facts.graph_plan().symbols);
    assert!(!facts.graph_plan().queue_project);
    assert_eq!(facts.stats.files_discovered, 2);
}

#[test]
fn explicitly_empty_graph_file_universe_is_complete() {
    let root = fixture_path("");
    let scoped = fixture_path("src/everything.tsx");
    let facts = collect_check_facts_with_graph_files_and_playwright(
        &root,
        vec![scoped],
        Vec::new(),
        CheckFactPlan::default(),
        None,
    );

    assert_eq!(
        crate::codebase::dependencies::graph::TsFactLookup::graph_files(&facts),
        Some([].as_slice())
    );
    assert!(facts.graph_file_universe_is_complete());
    assert!(facts.graph_file_universe().is_empty());
}

#[test]
fn collect_check_facts_only_parses_playwright_test_files_for_playwright_facts() {
    let root = fixture_path("");
    let test_file = fixture_path("src/everything.tsx");
    let invalid_file = fixture_path("src/invalid.ts");
    let facts = collect_check_facts_with_playwright(
        &root,
        vec![test_file.clone(), invalid_file.clone()],
        CheckFactPlan::default(),
        Some(playwright_plan(test_file.clone())),
    );

    assert_eq!(facts.stats.files_discovered, 2);
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(facts.stats.parse_errors, 0);
    assert!(facts
        .ts
        .get(&test_file)
        .expect("test file facts")
        .playwright
        .is_some());
    assert!(!facts.ts.contains_key(&invalid_file));
}

#[test]
fn playwright_fact_plan_union_preserves_staged_variants_and_source_metadata() {
    let first = fixture_path("src/everything.tsx");
    let second = fixture_path("src/widget.tsx");
    let mut plan = playwright_plan(first.clone());
    plan.set_source_files(vec![first.clone()]);
    let mut other = playwright_plan(second.clone());
    other.set_source_files(vec![second.clone()]);

    plan.include(other);

    assert!(plan.file(&first).is_some());
    assert!(plan.file(&second).is_some());
    assert_eq!(
        plan.source_files().as_ref(),
        &[
            crate::codebase::ts_resolver::normalize_path(&first),
            crate::codebase::ts_resolver::normalize_path(&second),
        ]
    );
}
