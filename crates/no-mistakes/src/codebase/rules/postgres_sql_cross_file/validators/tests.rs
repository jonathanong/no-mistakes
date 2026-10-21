use super::is_finite_validator;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    CallableId, ExportedBinding, ImportedBinding, ImportedBindingKind, ValueExpression,
};
use crate::codebase::dependencies::graph::test_support::{from_raw_maps, from_typed_maps};
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn member_callees_are_not_finite_validators() {
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let callee = ValueExpression::Member(
        std::sync::Arc::new(ValueExpression::Named {
            name: "schema".into(),
            scope: None,
            offset: 0,
            context: None,
        }),
        "toLowerCase".into(),
    );
    assert!(!is_finite_validator(
        &graph,
        &CheckFactMap::default(),
        &PathBuf::from("/repo/src/query.ts"),
        &callee
    ));
}

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: None,
        offset: 0,
        context: None,
    }
}

#[test]
fn imported_literal_helpers_and_parse_errors_are_classified() {
    let importer = PathBuf::from("/repo/src/query.ts");
    let producer = PathBuf::from("/repo/src/schema.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        importer.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: "./schema".into(),
                    local: "schemaOf".into(),
                    imported: "schemaOf".into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    facts.ts.insert(
        producer.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                exported_bindings: vec![ExportedBinding {
                    specifier: None,
                    local: "schemaOf".into(),
                    exported: "schemaOf".into(),
                }],
                callable_scope_ids: vec![(CallableId(1), "schemaOf".into())],
                value_flow: crate::codebase::dependencies::extract::ValueFlowFacts {
                    returns: vec![(CallableId(1), ValueExpression::Unknown)],
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    let graph = from_typed_maps(
        PathBuf::from("/repo"),
        [(
            NodeId::file(&importer),
            vec![(NodeId::file(&producer), EdgeKind::Import)],
        )]
        .into_iter()
        .collect(),
        Default::default(),
    );
    assert!(is_finite_validator(
        &graph,
        &facts,
        &importer,
        &named("schemaOf")
    ));
    facts.ts.insert(
        producer,
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                parse_error: Some("boom".into()),
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    assert!(!is_finite_validator(
        &graph,
        &facts,
        &importer,
        &named("schemaOf")
    ));
}
