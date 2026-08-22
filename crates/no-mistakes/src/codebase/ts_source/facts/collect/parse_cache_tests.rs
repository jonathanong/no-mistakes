use super::{
    collect_file_facts_with_sources_and_session,
    collect_ts_facts_with_context_sources_and_session_serializing_paths, TsFactContext, TsFactPlan,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn isolated_facts_dir() -> (tempfile::TempDir, PathBuf) {
    // Copy into a unique directory so begin_parse_count cannot pick up
    // parallel tests that parse the shared ast-snippets facts fixtures.
    let dir = tempfile::tempdir().unwrap();
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-source/fixture/facts");
    for name in ["imports.ts", "component.tsx"] {
        std::fs::copy(src.join(name), dir.path().join(name)).expect("copy facts fixture");
    }
    let root = crate::codebase::ts_resolver::normalize_path(dir.path());
    (dir, root)
}

fn fixture(root: &Path, name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&root.join(name))
}

fn sources_for(paths: &[PathBuf]) -> crate::codebase::ts_source::SourceStore {
    crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(paths),
    ))
}

fn collect_one(path: &Path, sources: &crate::codebase::ts_source::SourceStore, retain_parse: bool) {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_file_facts_with_sources_and_session(
        &session,
        path,
        TsFactPlan::imports(),
        &TsFactContext::default(),
        sources,
        retain_parse,
    );
}

#[test]
fn parallel_collect_ts_facts_evicts_request_parse_cache() {
    let (_dir, root) = isolated_facts_dir();
    let files = [
        fixture(&root, "imports.ts"),
        fixture(&root, "component.tsx"),
    ];
    crate::ast::with_request_parse_cache(|| {
        let facts = super::collect_ts_facts(&files, TsFactPlan::imports());
        assert_eq!(facts.len(), 2);
        assert_eq!(
            crate::ast::request_parse_cache_len(),
            0,
            "parallel fact collection evicts per-file parse cache entries"
        );
    });
}

#[test]
fn sequential_batch_evicts_after_serial_files_loop() {
    let (_dir, root) = isolated_facts_dir();
    let files = [
        fixture(&root, "imports.ts"),
        fixture(&root, "component.tsx"),
    ];
    let sources = sources_for(&files);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    crate::ast::with_request_parse_cache(|| {
        let facts = collect_ts_facts_with_context_sources_and_session_serializing_paths(
            &session,
            &files,
            TsFactPlan::imports(),
            &TsFactContext::default(),
            &sources,
            &files,
        );
        assert_eq!(facts.len(), 2);
        assert_eq!(crate::ast::request_parse_cache_len(), 0);
    });
}

#[test]
fn sequential_same_path_reuses_parse_until_evicted() {
    let (_dir, root) = isolated_facts_dir();
    let path = fixture(&root, "imports.ts");
    let sources = sources_for(std::slice::from_ref(&path));

    crate::ast::begin_parse_count(&root);
    crate::ast::with_request_parse_cache(|| {
        collect_one(&path, &sources, true);
        assert_eq!(crate::ast::request_parse_cache_len(), 1);
        collect_one(&path, &sources, true);
        assert_eq!(crate::ast::request_parse_cache_len(), 1);
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");

    crate::ast::begin_parse_count(&root);
    crate::ast::with_request_parse_cache(|| {
        collect_one(&path, &sources, true);
        crate::ast::evict_request_parse_cache_path(&path);
        assert_eq!(crate::ast::request_parse_cache_len(), 0);
        collect_one(&path, &sources, true);
        assert_eq!(crate::ast::request_parse_cache_len(), 1);
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(
        counts.get(&path),
        Some(&2),
        "evicting after the first collect forces a second physical parse: {counts:#?}"
    );
}
