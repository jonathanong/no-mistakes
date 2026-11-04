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
        vec![],
    );
    expect(
        vec![binding("col", named("unknown"))],
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

#[test]
fn outside_holes_drop_immutable_parameter_alias_chains() {
    expect(
        vec![binding("alias", named("uid"))],
        named("alias"),
        vec![],
        &[],
        vec![],
    );
    expect(
        vec![
            binding("first", named("uid")),
            binding("second", named("first")),
        ],
        named("second"),
        vec![],
        &[],
        vec![],
    );
}

#[test]
fn outside_aliases_keep_mutation_cycles_members_and_non_parameters() {
    let mut mutable = binding("alias", named("uid"));
    mutable.immutable = false;
    let mut invalidated = binding("alias", named("uid"));
    invalidated.invalidated_at = Some(3);
    for bindings in [
        vec![mutable],
        vec![invalidated],
        vec![
            binding("alias", named("cycle")),
            binding("cycle", named("alias")),
        ],
        vec![binding(
            "alias",
            ValueExpression::Member(Arc::new(named("uid")), "column".into()),
        )],
        vec![binding("alias", process_env("COLUMN"))],
    ] {
        expect(
            bindings,
            named("alias"),
            vec![],
            &[],
            vec![reference("alias")],
        );
    }
    let mut member = reference("alias");
    if let Provenance::Reference { members, .. } = &mut member {
        members.push("column".into());
    }
    expect(
        vec![binding("alias", named("uid"))],
        ValueExpression::Member(Arc::new(named("alias")), "column".into()),
        vec![],
        &[],
        vec![member],
    );
}
