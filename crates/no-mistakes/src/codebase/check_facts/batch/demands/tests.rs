use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::FamilyPathScope;
use crate::codebase::dependencies::extract::is_indexable;
use std::path::Path;

#[test]
fn batch_demands_share_primary_and_unmasked_graph_plans() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-include-scale")
        .canonicalize()
        .unwrap();
    let session = AnalysisSession::disabled();
    let snapshot = session.visible_paths(&root);
    let graph_files = [
        "shared/provider.mts",
        "shared/cycle.mts",
        "excluded/runtime-importer.mts",
    ]
    .map(|path| root.join(path));
    let files = snapshot
        .paths_for(&root)
        .iter()
        .filter(|path| is_indexable(path) && !graph_files.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    let [provider, cycle, importer] = &graph_files;
    let requests = [BatchCheckFactRequest {
        root: root.clone(),
        files,
        graph_files: graph_files.to_vec(),
        plan: CheckFactPlan {
            symbols: true,
            legacy_symbol_paths: [provider.clone()].into(),
            query_annotation: true,
            ..Default::default()
        },
        integration_discovery: FamilyPathScope::All,
        playwright: None,
        sources: snapshot.source_store_for(&root),
    }];
    let demands = demands_by_path(&requests);
    assert_eq!(requests[0].files.len(), 3073);
    let primary = &demands[&requests[0].files[0]][0].plan;
    assert!(requests[0]
        .files
        .iter()
        .all(|path| Arc::ptr_eq(primary, &demands[path][0].plan)));
    let graph = &demands[cycle][0].plan;
    assert!(Arc::ptr_eq(graph, &demands[importer][0].plan));
    assert!(!Arc::ptr_eq(primary, graph));
    assert!(!graph.query_annotation);
    assert!(!graph.symbols);
    let masked = &demands[provider][0].plan;
    assert!(!Arc::ptr_eq(graph, masked));
    assert!(masked.symbols);
    assert!(masked.legacy_symbol_paths.contains(provider));
}
