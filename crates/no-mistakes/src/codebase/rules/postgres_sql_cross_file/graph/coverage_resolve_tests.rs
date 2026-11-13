use super::coverage_harness::*;
use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueFlowFacts, ValueType,
};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;

#[test]
fn type_and_namespace_resolution_reject_missing_ambiguous_and_unreadable_files() {
    let root = PathBuf::from("/repo");
    let importer = root.join("src/query.ts");
    let left = root.join("src/left.ts");
    let right = root.join("src/right.ts");
    let mut facts = CheckFactMap::default();
    assert!(unique_type_export(
        &from_typed_maps(root.clone(), Default::default(), Default::default()),
        &facts,
        &importer,
        "Row"
    )
    .is_none());
    insert(
        &mut facts,
        &importer,
        TsFileFacts {
            imported_bindings: vec![ImportedBinding {
                is_type_only: true,
                ..named_import("Row", "Row", "./left")
            }],
            ..Default::default()
        },
    );
    let empty = from_typed_maps(root.clone(), Default::default(), Default::default());
    assert!(unique_type_export(&empty, &facts, &importer, "Missing").is_none());
    assert!(unique_type_export(&empty, &facts, &importer, "Row").is_none());
    insert(
        &mut facts,
        &left,
        TsFileFacts {
            value_flow: ValueFlowFacts {
                type_exports: vec![export(None, "Row", "Row")],
                type_shapes: [("Row".into(), ValueType::Element("TABLES".into()))]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    insert(
        &mut facts,
        &right,
        TsFileFacts {
            value_flow: ValueFlowFacts {
                type_exports: vec![export(None, "Row", "Row")],
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let forward = [(
        NodeId::file(&importer),
        vec![
            (NodeId::file(&left), EdgeKind::Import),
            (NodeId::file(&right), EdgeKind::Import),
        ],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(root.clone(), forward, Default::default());
    assert!(unique_type_export(&graph, &facts, &importer, "Row").is_none());
    assert!(
        unique_namespace_member(&graph, &facts, &root.join("src/absent.ts"), "ns", "Row").is_none()
    );
    facts.ts.insert(
        importer.clone(),
        file_facts(TsFileFacts {
            operational_error: Some("unreadable".into()),
            imported_bindings: vec![ImportedBinding {
                kind: ImportedBindingKind::Namespace,
                ..named_import("ns", "*", "./left")
            }],
            ..Default::default()
        }),
    );
    assert!(unique_namespace_member(&graph, &facts, &importer, "ns", "Row").is_none());
}
