use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::dependencies::graph::test_support::{from_raw_maps, from_typed_maps};
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn file_facts(ts: TsFileFacts) -> Arc<CheckFileFacts> {
    CheckFileFacts {
        ts: Arc::new(ts),
        ..Default::default()
    }
    .into()
}

fn named_import(local: &str) -> ImportedBinding {
    ImportedBinding {
        specifier: "./tables".into(),
        local: local.into(),
        imported: "tables".into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    }
}

fn local_binding(name: &str) -> ExportedBinding {
    ExportedBinding {
        specifier: None,
        local: name.into(),
        exported: name.into(),
    }
}

#[test]
fn graph_plan_requires_a_configured_sql_cross_file_rule() {
    assert!(graph_plan(&crate::config::v2::NoMistakesConfig::default()).is_none());
}

#[test]
fn unique_export_abstains_on_unproven_or_ambiguous_producers() {
    let importer = PathBuf::from("/repo/src/query.ts");
    let producer = PathBuf::from("/repo/src/tables.ts");
    let other = PathBuf::from("/repo/src/other.ts");
    let missing = PathBuf::from("/repo/src/missing.ts");
    let importer_node = NodeId::file(&importer);
    let producer_node = NodeId::file(&producer);
    let other_node = NodeId::file(&other);
    let missing_node = NodeId::file(&missing);
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            parse_error: Some("boom".into()),
            imported_bindings: vec![named_import("tables")],
            ..Default::default()
        }),
    );
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    assert!(unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &importer,
        "tables"
    )
    .is_none());

    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![named_import("tables")],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        producer.clone(),
        file_facts(TsFileFacts {
            parse_error: Some("producer boom".into()),
            exported_bindings: vec![local_binding("tables")],
            ..Default::default()
        }),
    );
    let forward = [(
        importer_node.clone(),
        vec![
            (producer_node.clone(), EdgeKind::Import),
            (other_node.clone(), EdgeKind::Call),
            (missing_node.clone(), EdgeKind::Import),
            (NodeId::module("pkg"), EdgeKind::Import),
        ],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    assert!(unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &importer,
        "tables"
    )
    .is_none());

    facts.ts.insert(
        producer.clone(),
        file_facts(TsFileFacts {
            exported_bindings: vec![local_binding("tables")],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        other.clone(),
        file_facts(TsFileFacts {
            exported_bindings: vec![local_binding("tables")],
            ..Default::default()
        }),
    );
    let forward = [(
        importer_node,
        vec![
            (producer_node, EdgeKind::Import),
            (other_node, EdgeKind::Import),
        ],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    assert!(unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &importer,
        "tables"
    )
    .is_none());

    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![ImportedBinding {
                specifier: "./tables".into(),
                local: "t".into(),
                imported: "*".into(),
                kind: ImportedBindingKind::Namespace,
                is_type_only: false,
            }],
            ..Default::default()
        }),
    );
    assert!(unique_export(&graph, &facts, &RegistryRoutes::new(&facts), &importer, "t").is_none());
}

#[test]
fn unique_export_follows_named_reexports_stars_and_namespace_members() {
    let importer = PathBuf::from("/repo/src/query.ts");
    let barrel = PathBuf::from("/repo/src/barrel.ts");
    let producer = PathBuf::from("/repo/src/tables.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![ImportedBinding {
                specifier: "./barrel".into(),
                local: "registry".into(),
                imported: "registry".into(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            }],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        barrel.clone(),
        file_facts(TsFileFacts {
            exported_bindings: vec![ExportedBinding {
                specifier: Some("./tables".into()),
                local: "tables".into(),
                exported: "registry".into(),
            }],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        producer.clone(),
        file_facts(TsFileFacts {
            exported_bindings: vec![local_binding("tables")],
            ..Default::default()
        }),
    );
    let forward = [
        (
            NodeId::file(&importer),
            vec![(NodeId::file(&barrel), EdgeKind::Import)],
        ),
        (
            NodeId::file(&barrel),
            vec![(NodeId::file(&producer), EdgeKind::Import)],
        ),
    ]
    .into_iter()
    .collect();
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    let unique = unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &importer,
        "registry",
    )
    .unwrap();
    assert_eq!(unique.producer, producer);
    assert_eq!(unique.exported, "registry");

    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![named_import("tables")],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        barrel.clone(),
        file_facts(TsFileFacts {
            star_reexport_specifiers: vec!["./tables".into()],
            ..Default::default()
        }),
    );
    let unique = unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &importer,
        "tables",
    )
    .unwrap();
    assert_eq!(unique.producer, producer);
    assert_eq!(unique.exported, "tables");

    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![ImportedBinding {
                specifier: "./tables".into(),
                local: "t".into(),
                imported: "*".into(),
                kind: ImportedBindingKind::Namespace,
                is_type_only: false,
            }],
            ..Default::default()
        }),
    );
    let forward = [(
        NodeId::file(&importer),
        vec![(NodeId::file(&producer), EdgeKind::Import)],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    let unique = unique_namespace_member(&graph, &facts, &importer, "t", "tables").unwrap();
    assert_eq!(unique.producer, producer);
    assert_eq!(unique.exported, "tables");
    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![named_import("tables")],
            ..Default::default()
        }),
    );
    assert!(unique_namespace_member(&graph, &facts, &importer, "tables", "users").is_none());

    let left = PathBuf::from("/repo/src/left.ts");
    let right = PathBuf::from("/repo/src/right.ts");
    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            imported_bindings: vec![ImportedBinding {
                specifier: "./left".into(),
                local: "registry".into(),
                imported: "registry".into(),
                kind: ImportedBindingKind::Named,
                is_type_only: false,
            }],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        left.clone(),
        file_facts(TsFileFacts {
            exported_bindings: vec![ExportedBinding {
                specifier: Some("./right".into()),
                local: "registry".into(),
                exported: "registry".into(),
            }],
            ..Default::default()
        }),
    );
    facts.ts.insert(
        right.clone(),
        file_facts(TsFileFacts {
            exported_bindings: vec![ExportedBinding {
                specifier: Some("./left".into()),
                local: "registry".into(),
                exported: "registry".into(),
            }],
            ..Default::default()
        }),
    );
    let forward = [
        (
            NodeId::file(&importer),
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
    let graph = from_typed_maps(PathBuf::from("/repo"), forward, Default::default());
    assert!(unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &importer,
        "registry"
    )
    .is_none());
}
