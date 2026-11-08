use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};
use std::sync::Arc;

#[test]
fn legacy_projection_attributes_value_flow_without_extra_parses_or_reads() {
    assert_fallback("legacy.source");
}

#[test]
fn recovery_already_attributes_value_flow_without_extra_parses_or_reads() {
    assert_fallback("recovery.ts");
}

fn assert_fallback(name: &str) {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/diagnostics-fallback/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join(name);
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new_observed(
        Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
            std::slice::from_ref(&path),
        )),
        Some(Arc::clone(&observer)),
    ));
    let plan = CheckFactPlan {
        imports: true,
        production_import_paths: Arc::new([path.clone()].into()),
        ..Default::default()
    };
    crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
        let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
            &session,
            root,
            (vec![path.clone()], Vec::new()),
            plan,
            None,
            sources,
        );
        let file = facts.ts.get(&path).unwrap();
        assert!(file.parsed);
        assert_eq!(file.parse_error.is_some(), name == "recovery.ts");
    });
    let work = session.work_snapshot();
    assert_eq!(work.source_reads.len(), 1, "{work:?}");
    assert_eq!(work.parse_attempts.len(), 1, "{work:?}");
    assert_eq!(work.source_reads.get(&path), Some(&1), "{work:?}");
    assert_eq!(work.parse_attempts.get(&path), Some(&1), "{work:?}");
    let rows = observer.hot_file_snapshot();
    for pass in [FilePass::ValueFlowExtract, FilePass::ValueFlowFinalize] {
        assert!(
            rows.iter()
                .any(|row| row.pass == pass && row.path == path.to_string_lossy()),
            "{name}: missing {pass:?}: {rows:?}"
        );
    }
}
