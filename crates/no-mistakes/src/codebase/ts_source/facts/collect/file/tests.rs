use super::*;
use crate::codebase::ts_source::facts::{TsFactContext, TsFactPlan};
use std::path::PathBuf;

#[test]
fn facts_from_collection_result_keeps_both_error_channels() {
    let facts = facts_from_collection_result(Err(anyhow::anyhow!("oxc drifted")));
    assert_eq!(facts.operational_error.as_deref(), Some("oxc drifted"));
    assert_eq!(facts.parse_error.as_deref(), Some("oxc drifted"));
}

#[test]
fn collect_file_facts_reports_source_store_read_failures() {
    let path = PathBuf::from("/missing-collect-file.ts");
    let inventory =
        crate::codebase::ts_source::FileInventory::from_paths(std::slice::from_ref(&path));
    let sources = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(inventory));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = collect_file_facts_with_sources_and_session(
        &session,
        &path,
        TsFactPlan::imports(),
        &TsFactContext::default(),
        &sources,
        false,
    )
    .expect("read failures still produce facts");
    assert!(facts
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
}

#[test]
fn collect_file_facts_uses_the_typescript_fallback_for_unknown_extensions() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    let path = crate::codebase::ts_resolver::normalize_path(&path);
    let inventory =
        crate::codebase::ts_source::FileInventory::from_paths(std::slice::from_ref(&path));
    let sources = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(inventory));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = collect_file_facts_with_sources_and_session(
        &session,
        &path,
        TsFactPlan {
            source: true,
            ..TsFactPlan::default()
        },
        &TsFactContext::default(),
        &sources,
        true,
    )
    .expect("markdown is parsed through the TypeScript fallback");
    assert!(facts.source.is_some());
}

#[test]
fn exact_argument_projection_skips_unselected_saved_calls() {
    use crate::codebase::ts_source::facts::ArgumentProjection;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let root = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let path = root.join("exact-projection.mjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());
    let plan = TsFactPlan {
        imports: true,
        function_calls: true,
        call_arguments: true,
        ..Default::default()
    };
    let mut context = TsFactContext::new(&root);
    context.argument_projection = ArgumentProjection::exact(vec!["selected".into()]);
    crate::codebase::ts_source::module_facts::arguments::probe::reset_projected_calls();
    let selected = collect_file_facts_from_program(
        &path,
        plan,
        &context,
        &source,
        &parsed.program,
        None,
        None,
    );
    assert_eq!(
        crate::codebase::ts_source::module_facts::arguments::probe::projected_calls(),
        2
    );
    let selected = selected.module_bindings.unwrap();
    assert_eq!(selected.call_arguments.as_ref().unwrap().len(), 2);

    crate::codebase::ts_source::module_facts::arguments::probe::reset_projected_calls();
    let broad = collect_file_facts_from_program(
        &path,
        plan,
        &TsFactContext::new(&root),
        &source,
        &parsed.program,
        None,
        None,
    );
    assert_eq!(
        crate::codebase::ts_source::module_facts::arguments::probe::projected_calls(),
        4
    );
    let broad = broad.module_bindings.unwrap();
    for (offset, values) in selected.call_arguments.as_ref().unwrap().iter() {
        assert_eq!(
            broad.call_arguments.as_ref().unwrap().get(offset),
            Some(values)
        );
    }
}
