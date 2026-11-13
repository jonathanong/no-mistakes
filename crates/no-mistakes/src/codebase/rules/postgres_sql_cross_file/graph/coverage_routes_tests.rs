use super::coverage_harness::*;
use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    ImportKind, ImportedBinding, ImportedBindingKind, ValueBinding, ValueExpression, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;

#[test]
fn registry_routes_reject_missing_files_broken_reexports_and_require_shapes() {
    let root = PathBuf::from("/repo");
    let consumer = root.join("src/consumer.ts");
    let left = root.join("src/left.ts");
    let right = root.join("src/right.ts");
    let ghost = root.join("src/ghost.ts");
    let mut facts = CheckFactMap::default();
    insert(
        &mut facts,
        &consumer,
        TsFileFacts {
            imports: vec![import("./left", ImportKind::Require, false)],
            eager_require_specifiers: vec!["./left".into(), "./right".into()],
            value_flow: ValueFlowFacts {
                bindings: vec![
                    require_binding("cycled", 10),
                    require_binding("broken", 20),
                    ValueBinding {
                        value: ValueExpression::Unknown,
                        ..require_binding("plain", 30)
                    },
                    require_binding("unlisted", 40),
                    require_binding("dynamic", 50),
                    require_binding("lazy", 60),
                ],
                ..Default::default()
            },
            function_calls: vec![
                require_call(10, Some("./left")),
                require_call(20, Some("./left")),
                require_call(50, None),
                require_call(60, Some("./lazy")),
            ],
            ..Default::default()
        },
    );
    insert(
        &mut facts,
        &left,
        TsFileFacts {
            imports: vec![import("./right", ImportKind::Static, false)],
            exported_bindings: vec![
                export(Some("./right"), "registry", "registry"),
                export(Some("./missing"), "tables", "broken"),
                export(None, "*", "group"),
            ],
            ..Default::default()
        },
    );
    insert(
        &mut facts,
        &right,
        TsFileFacts {
            imports: vec![import("./left", ImportKind::Static, false)],
            exported_bindings: vec![export(Some("./left"), "registry", "registry")],
            ..Default::default()
        },
    );
    let forward = [
        (
            NodeId::file(&consumer),
            vec![(NodeId::file(&left), EdgeKind::Require)],
        ),
        (
            NodeId::file(&left),
            vec![(NodeId::file(&right), EdgeKind::Import)],
        ),
        (
            NodeId::file(&right),
            vec![(NodeId::file(&left), EdgeKind::Import)],
        ),
    ]
    .into_iter()
    .collect();
    let graph = from_typed_maps(root, forward, Default::default());
    let routes = RegistryRoutes::new(&facts);
    let members = |name: &str| vec![name.to_string()];
    assert!(registry_route(&graph, &routes, &ghost, "cycled", &members("registry")).is_none());
    assert!(registry_route(&graph, &routes, &consumer, "cycled", &members("registry")).is_none());
    assert!(registry_route(&graph, &routes, &consumer, "broken", &members("broken")).is_none());
    assert!(registry_route(&graph, &routes, &consumer, "plain", &[]).is_none());
    assert!(registry_route(&graph, &routes, &consumer, "unlisted", &[]).is_none());
    assert!(registry_route(&graph, &routes, &consumer, "dynamic", &[]).is_none());
    assert!(registry_route(&graph, &routes, &consumer, "lazy", &[]).is_none());

    facts.ts.insert(
        consumer.clone(),
        file_facts(TsFileFacts {
            imports: vec![import("./left", ImportKind::Static, false)],
            imported_bindings: vec![named_import("group", "group", "./left")],
            ..Default::default()
        }),
    );
    let forward = [(
        NodeId::file(&consumer),
        vec![(NodeId::file(&left), EdgeKind::Import)],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    let routes = RegistryRoutes::new(&facts);
    assert!(registry_route(&graph, &routes, &consumer, "group", &members("TABLES")).is_none());

    facts.ts.insert(
        consumer.clone(),
        file_facts(TsFileFacts {
            imports: vec![import("./ghost", ImportKind::Static, false)],
            imported_bindings: vec![named_import("direct", "direct", "./ghost")],
            ..Default::default()
        }),
    );
    let forward = [(
        NodeId::file(&consumer),
        vec![(NodeId::file(&ghost), EdgeKind::Import)],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    let routes = RegistryRoutes::new(&facts);
    assert!(registry_route(&graph, &routes, &consumer, "direct", &members("TABLES")).is_none());
    facts.ts.insert(
        consumer.clone(),
        file_facts(TsFileFacts {
            imports: vec![import("./ghost", ImportKind::Static, false)],
            imported_bindings: vec![ImportedBinding {
                kind: ImportedBindingKind::Namespace,
                ..named_import("ns", "*", "./ghost")
            }],
            ..Default::default()
        }),
    );
    let routes = RegistryRoutes::new(&facts);
    assert!(registry_route(&graph, &routes, &consumer, "ns", &members("TABLES")).is_none());
}
