use super::*;
use crate::config::v2::NoMistakesConfig;

#[test]
fn local_caller_entries_skip_failed_fact_entries_without_symbols() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );
    let failed_path = root.join("failed.mts");
    let facts = TsFactMap::from([(
        failed_path.clone(),
        crate::codebase::ts_source::facts::TsFileFacts {
            operational_error: Some("synthetic read failure".to_string()),
            ..Default::default()
        },
    )]);
    let workspace = crate::codebase::workspaces::load_from_files(&root, &[]).unwrap();
    let visible_files = [failed_path].into_iter().collect::<crate::fx::PathSet>();
    let remapper = crate::codebase::ts_source::FrozenPathRemapper::from_paths(
        visible_files.iter().cloned(),
    );
    let context = prepare_local_caller_context(&facts, &workspace, &visible_files, &remapper);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig(None, &root).unwrap();
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        &tsconfig,
        Some(&visible_files),
        &session,
    );
    let filter = TestFileFilter::new(&root, &NoMistakesConfig::default());

    let callers = local_caller_entries(
        &context,
        &BTreeMap::new(),
        &root,
        &resolver,
        &filter,
        false,
    );

    assert!(callers.is_empty());
}
