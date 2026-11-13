use super::coverage_harness::*;
use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;

#[test]
fn route_targets_skip_non_file_edges_and_reject_unresolvable_imports() {
    let root = PathBuf::from("/repo");
    let mixed = root.join("src/mixed.ts");
    let catalog = root.join("src/catalog.ts");
    let lonely = root.join("src/lonely.ts");
    let computed = root.join("src/computed.ts");
    let mut facts = CheckFactMap::default();
    insert(
        &mut facts,
        &mixed,
        TsFileFacts {
            imports: vec![import("./catalog", ImportKind::Static, false)],
            ..Default::default()
        },
    );
    insert(&mut facts, &catalog, TsFileFacts::default());
    insert(&mut facts, &lonely, TsFileFacts::default());
    insert(
        &mut facts,
        &computed,
        TsFileFacts {
            imports: vec![import("./catalog", ImportKind::Static, true)],
            ..Default::default()
        },
    );
    let forward = [
        (
            NodeId::file(&mixed),
            vec![
                (NodeId::file(&catalog), EdgeKind::Import),
                (
                    NodeId::queue_job(root.join("src/jobs.ts"), "send"),
                    EdgeKind::Import,
                ),
            ],
        ),
        (
            NodeId::file(&computed),
            vec![(NodeId::file(&catalog), EdgeKind::Import)],
        ),
    ]
    .into_iter()
    .collect();
    let graph = from_typed_maps(root.clone(), forward, Default::default());
    let routes = RegistryRoutes::new(&facts);
    assert!(routes.file(&catalog).is_some());
    assert!(routes.file(&root.join("src/../src/catalog.ts")).is_none());
    let resolved = routes
        .target(&graph, &mixed, "./catalog", ImportKind::Static)
        .expect("queue-job edge is not a second file");
    assert_eq!(resolved, catalog);
    assert!(routes
        .target(&graph, &lonely, "./catalog", ImportKind::Static)
        .is_none());
    assert!(routes
        .target(&graph, &computed, "./catalog", ImportKind::Static)
        .is_none());
    assert!(routes
        .target(
            &graph,
            &root.join("src/absent.ts"),
            "./catalog",
            ImportKind::Static
        )
        .is_none());
}
