use super::{export_is_unbounded, identifier_finding};

// Output cases moved to cli_postgres_snapshot_no_unbounded_sql_identifier.
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{Provenance, SqlCall};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn imported_config_without_prepared_producer_facts_is_not_unbounded() {
    assert!(!export_is_unbounded(
        &CheckFactMap::default(),
        Path::new("/repo/src/config.ts"),
        "config"
    ));
}

#[test]
fn same_file_or_literal_exports_are_not_unbounded_identifier_sources() {
    let file = PathBuf::from("/repo/src/query.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: "./query".into(),
                    local: "schema".into(),
                    imported: "schema".into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
                exported_bindings: vec![ExportedBinding {
                    specifier: None,
                    local: "schema".into(),
                    exported: "schema".into(),
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    let mut forward = HashMap::new();
    forward.insert(file.clone(), vec![file.clone()]);
    let graph = from_raw_maps(PathBuf::from("/repo"), forward, Default::default());
    let owned = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (0, 1),
        call: &owned,
    };
    assert!(identifier_finding(
        Path::new("/repo"),
        &graph,
        &facts,
        &super::RegistryRoutes::new(&facts),
        &call,
        &Provenance::Reference {
            name: "schema".into(),
            scope: 0,
            offset: 0,
            members: vec![]
        },
    )
    .is_none());
}

#[test]
fn shadowed_process_and_unbounded_namespace_members_are_classified() {
    let file = PathBuf::from("/repo/src/query.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: "./process".into(),
                    local: "process".into(),
                    imported: "process".into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let owned = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (0, 1),
        call: &owned,
    };
    assert!(identifier_finding(
        Path::new("/repo"),
        &graph,
        &facts,
        &super::RegistryRoutes::new(&facts),
        &call,
        &Provenance::ProcessEnv("SCHEMA".into()),
    )
    .is_none());
    assert!(identifier_finding(
        Path::new("/repo"),
        &graph,
        &facts,
        &super::RegistryRoutes::new(&facts),
        &call,
        &Provenance::Namespace {
            local: "t".into(),
            exported: "config".into(),
            members: vec!["config".into()],
        },
    )
    .is_none());
}
