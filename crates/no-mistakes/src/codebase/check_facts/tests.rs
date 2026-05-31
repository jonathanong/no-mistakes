use super::{
    collect_check_facts, collect_check_facts_with_graph_files_and_playwright,
    collect_check_facts_with_playwright, collect_file_facts, CheckFactPlan, PlaywrightFactPlan,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

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
    let file_facts = facts.ts.get(&file).expect("file facts are collected");
    assert!(!file_facts.ts.imports.is_empty());
    assert!(file_facts.symbols.is_some());
    assert!(file_facts.react.is_some());
    assert!(file_facts.ts.queue_project.is_some());
    assert!(file_facts.integration.is_some());
    assert!(file_facts.nextjs_caching.is_some());
    assert!(file_facts.dynamic_imports.is_some());
    assert!(file_facts.source.is_some());
}

#[test]
fn collect_check_facts_parses_once_for_playwright_and_shared_facts() {
    let root = fixture_path("");
    let file = fixture_path("src/everything.tsx");
    let mut test_id_attributes_by_path = HashMap::new();
    test_id_attributes_by_path.insert(file.clone(), vec!["data-testid".to_string()]);

    let facts = collect_check_facts_with_playwright(
        &root,
        vec![file.clone()],
        CheckFactPlan {
            react: true,
            ..CheckFactPlan::default()
        },
        Some(PlaywrightFactPlan {
            navigation_helpers: Vec::new(),
            selector_regexes: Arc::new(
                crate::playwright::selectors::compile_selector_regexes_with_html_ids(
                    &["data-testid".to_string()],
                    &Default::default(),
                    false,
                ),
            ),
            test_id_attributes_by_path: Arc::new(test_id_attributes_by_path),
        }),
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
    assert_eq!(facts.graph_files(), std::slice::from_ref(&graph_only));
    assert!(facts.ts.contains_key(&scoped));
    assert!(facts.ts.contains_key(&graph_only));
    let graph_only_facts = facts.ts.get(&graph_only).expect("graph-only facts");
    assert!(!graph_only_facts.ts.imports.is_empty());
    assert!(graph_only_facts.ts.queue_project.is_none());
    assert!(graph_only_facts.integration.is_none());
    assert_eq!(facts.stats.files_discovered, 2);
}

#[test]
fn collect_check_facts_only_parses_playwright_test_files_for_playwright_facts() {
    let root = fixture_path("");
    let test_file = fixture_path("src/everything.tsx");
    let invalid_file = fixture_path("src/invalid.ts");
    let mut test_id_attributes_by_path = HashMap::new();
    test_id_attributes_by_path.insert(test_file.clone(), vec!["data-testid".to_string()]);

    let facts = collect_check_facts_with_playwright(
        &root,
        vec![test_file.clone(), invalid_file.clone()],
        CheckFactPlan::default(),
        Some(PlaywrightFactPlan {
            navigation_helpers: Vec::new(),
            selector_regexes: Arc::new(
                crate::playwright::selectors::compile_selector_regexes_with_html_ids(
                    &["data-testid".to_string()],
                    &Default::default(),
                    false,
                ),
            ),
            test_id_attributes_by_path: Arc::new(test_id_attributes_by_path),
        }),
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
    assert!(
        !facts
            .ts
            .get(&invalid_file)
            .expect("non-playwright file facts")
            .parsed
    );
}
