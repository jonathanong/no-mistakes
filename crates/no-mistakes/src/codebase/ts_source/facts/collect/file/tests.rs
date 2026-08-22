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
