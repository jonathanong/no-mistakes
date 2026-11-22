use super::*;

#[test]
fn leftover_sql_demands_share_one_mask_per_original_plan() {
    let root = fixture();
    let session = AnalysisSession::disabled();
    let snapshot = session.visible_paths(&root);
    let sources = snapshot.source_store_for(&root);
    let files = snapshot
        .paths_for(&root)
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .cloned()
        .collect::<Vec<_>>();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let primary = Arc::new(prepared(&root, &files, &config));
    let broken = root.join("unrelated/broken.mts");
    let graph = Arc::new(CheckFactPlan {
        symbols: true,
        legacy_symbol_paths: [broken.clone()].into(),
        ..Default::default()
    });
    let mut demands = files
        .iter()
        .map(|path| {
            let plan = if *path == broken { &graph } else { &primary };
            (
                path.clone(),
                vec![FactDemand {
                    request: 0,
                    plan: Arc::clone(plan),
                }],
            )
        })
        .collect();
    let selected = root.join("unrelated/dense.mts");
    let requests = [BatchCheckFactRequest {
        integration_discovery: crate::codebase::check_facts::FamilyPathScope::All,
        root: root.clone(),
        files,
        graph_files: vec![],
        plan: (*primary).clone(),
        playwright: None,
        sources,
    }];
    let collected = collect(
        &session,
        &requests,
        &mut demands,
        Some([selected.clone()].into()),
    );
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].0, selected);
    assert!(Arc::ptr_eq(&collected[0].1[0].plan, &primary));
    assert!(primary.embedded_sql);

    let remaining = &demands[&root.join("src/caller.mts")][0].plan;
    let helper = &demands[&root.join("helpers/paging.mts")][0].plan;
    assert!(Arc::ptr_eq(remaining, helper));
    assert!(!remaining.embedded_sql && remaining.embedded_sql_options.is_empty());
    assert!(remaining.graph.imports);
    assert!(!remaining.symbols);
    let legacy = &demands[&broken][0].plan;
    assert!(!Arc::ptr_eq(remaining, legacy));
    assert!(legacy.symbols && legacy.legacy_symbol_paths.contains(&broken));
}
