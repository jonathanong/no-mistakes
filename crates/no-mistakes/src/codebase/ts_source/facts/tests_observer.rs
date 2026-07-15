use super::*;
use std::path::PathBuf;

#[test]
fn parallel_collection_deduplicates_successful_and_failed_parse_work() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parser-count/parallel-dedup");
    let valid = root.join("valid.mts");
    let syntax_error = root.join("syntax-error.mts");
    // Keep later-sorting lexical aliases in the input so normalized-path
    // deduplication preserves the deterministic original keys callers expect.
    let valid_alias = root.join("z-alias").join("..").join("valid.mts");
    let syntax_error_alias = root.join("z-alias").join("..").join("syntax-error.mts");
    let files = (0..64)
        .flat_map(|_| {
            [
                valid.clone(),
                valid_alias.clone(),
                syntax_error.clone(),
                syntax_error_alias.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        &files,
    ));
    let sources = crate::codebase::ts_source::SourceStore::new_observed(
        inventory,
        Some(std::sync::Arc::clone(&observer)),
    );

    let facts = super::super::collect::collect_ts_facts_with_context_sources_and_session(
        &session,
        &files,
        TsFactPlan::imports(),
        &TsFactContext::default(),
        &sources,
    );

    assert_eq!(facts.len(), 2);
    assert!(facts[&valid].parse_error.is_none());
    assert!(!facts.contains_key(&valid_alias));
    assert!(!facts.contains_key(&syntax_error_alias));
    assert!(facts[&syntax_error].parse_error.is_some());
    assert_eq!(sources.physical_read_count(), 2);
    let work = session.work_snapshot();
    assert_eq!(work.source_reads.len(), 2);
    assert_eq!(
        work.source_reads[&crate::codebase::ts_resolver::normalize_path(&valid)],
        1
    );
    assert_eq!(
        work.source_reads[&crate::codebase::ts_resolver::normalize_path(&syntax_error)],
        1
    );
    assert_eq!(
        work.parse_attempts[&crate::codebase::ts_resolver::normalize_path(&valid)],
        1
    );
    assert_eq!(
        work.parse_attempts[&crate::codebase::ts_resolver::normalize_path(&syntax_error)],
        1
    );
    let diagnostics = observer.snapshot().work;
    assert_eq!(diagnostics["source.requests"], 2);
    assert_eq!(diagnostics["parse.requests"], 2);
    assert_eq!(diagnostics["parse.files"], 2);
    assert_eq!(diagnostics["parse.errors"], 1);
}
