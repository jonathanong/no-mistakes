use super::super::Provenance;
use super::harness::{binding, expect, named, process_env, reference};
use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueExpression,
};
use std::sync::Arc;

#[test]
fn outside_holes_keep_env_config_and_drop_parameters() {
    expect(
        vec![],
        process_env("WCOL"),
        vec![],
        &[],
        vec![Provenance::ProcessEnv("WCOL".into())],
    );
    expect(
        vec![binding("col", process_env("WCOL"))],
        named("col"),
        vec![],
        &[],
        vec![reference("col")],
    );
    expect(
        vec![binding("row", ValueExpression::Call(30, 40))],
        named("row"),
        vec![],
        &[(30, 40)],
        vec![reference("row")],
    );
    expect(
        vec![binding("col", named("free"))],
        named("col"),
        vec![],
        &[],
        vec![reference("col")],
    );
    expect(vec![], named("uid"), vec![], &[], vec![]);
    expect(
        vec![binding("flag", ValueExpression::Unknown)],
        named("flag"),
        vec![],
        &[],
        vec![reference("flag")],
    );
    expect(
        vec![],
        ValueExpression::Member(Arc::new(named("ns")), "table".into()),
        vec![ImportedBinding {
            specifier: "./tables".into(),
            local: "ns".into(),
            imported: "*".into(),
            kind: ImportedBindingKind::Namespace,
            is_type_only: false,
        }],
        &[],
        vec![Provenance::Namespace {
            local: "ns".into(),
            exported: "table".into(),
            members: vec!["table".into()],
        }],
    );
    expect(
        vec![],
        named("plain"),
        vec![ImportedBinding {
            specifier: "./config".into(),
            local: "plain".into(),
            imported: "plain".into(),
            kind: ImportedBindingKind::Named,
            is_type_only: false,
        }],
        &[],
        vec![reference("plain")],
    );
}
