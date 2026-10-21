use super::env::{env_from_chain, env_key};
use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueExpression,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: None,
        offset: 0,
        context: None,
    }
}

fn member(inner: ValueExpression, key: &str) -> ValueExpression {
    ValueExpression::Member(Arc::new(inner), key.into())
}

#[test]
fn process_env_requires_the_process_env_member_chain() {
    assert!(is_process_env(&member(
        member(named("process"), "env"),
        "SCHEMA"
    )));
    assert!(!is_process_env(&member(
        member(named("process"), "argv"),
        "0"
    )));
    assert!(!is_process_env(&member(
        member(named("config"), "env"),
        "SCHEMA"
    )));
}

#[test]
fn process_specifiers_include_node_and_bare_process() {
    assert!(is_process_specifier("node:process"));
    assert!(is_process_specifier("process"));
    assert!(!is_process_specifier("./process"));
}

fn imported(local: &str, imported: &str, specifier: &str) -> CheckFactMap {
    let file = PathBuf::from("/repo/src/query.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file,
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: specifier.into(),
                    local: local.into(),
                    imported: imported.into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    facts
}

#[test]
fn imported_process_and_env_aliases_are_process_env_keys() {
    let file = PathBuf::from("/repo/src/query.ts");
    let process = imported("proc", "process", "node:process");
    assert_eq!(
        env_key(
            &process,
            &file,
            &member(member(named("proc"), "env"), "SCHEMA")
        ),
        Some("SCHEMA")
    );
    assert_eq!(
        env_from_chain(
            &process,
            &file,
            &named("proc"),
            &["env".into(), "SCHEMA".into(), "toLowerCase".into()]
        ),
        Some("SCHEMA".into())
    );
    let env = imported("env", "env", "process");
    assert_eq!(
        env_key(&env, &file, &member(named("env"), "SCHEMA")),
        Some("SCHEMA")
    );
    assert_eq!(
        env_from_chain(&env, &file, &named("env"), &["SCHEMA".into()]),
        Some("SCHEMA".into())
    );
    assert!(env_from_chain(&env, &file, &ValueExpression::Unknown, &["SCHEMA".into()]).is_none());
    assert!(env_key(&process, &file, &named("proc")).is_none());
    assert!(env_key(&process, &file, &member(member(named("proc"), "argv"), "0")).is_none());
}

#[test]
fn opaque_and_projected_fact_containers_preserve_input_references() {
    let file = PathBuf::from("/repo/src/query.ts");
    let mut facts = imported("ns", "*", "./tables");
    Arc::make_mut(&mut Arc::get_mut(facts.ts.get_mut(&file).unwrap()).unwrap().ts)
        .imported_bindings[0]
        .kind = ImportedBindingKind::Namespace;
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let bindings = Bindings::new(&facts);
    let mut fields = crate::fx::fx_map();
    fields.insert("schema".into(), named("config"));
    // These typed containers exercise provenance independently of runtime array semantics.
    for expression in [
        ValueExpression::Awaited(Arc::new(named("config"))),
        member(
            ValueExpression::Derived(vec![named("config")].into()),
            "schema",
        ),
        ValueExpression::Object(Arc::new(fields.clone()), false),
        ValueExpression::Array(Arc::new(fields)),
        ValueExpression::Either(Arc::new(named("config")), Arc::new(named("config"))),
    ] {
        let mut found = BTreeSet::new();
        classify(&graph, &facts, &file, &expression, &mut found, &bindings);
        assert_eq!(
            found,
            BTreeSet::from([Provenance::Reference {
                name: "config".into(),
                scope: 0,
                offset: 0,
                members: Vec::new(),
            }])
        );
    }
    let mut found = BTreeSet::new();
    classify(&graph, &facts, &file, &named("ns"), &mut found, &bindings);
    assert_eq!(
        found,
        BTreeSet::from([Provenance::Reference {
            name: "ns".into(),
            scope: 0,
            offset: 0,
            members: Vec::new(),
        }])
    );
}

#[test]
fn alias_roots_require_global_or_import_binding_identity() {
    let file = PathBuf::from("/repo/src/query.ts");
    let facts = CheckFactMap::default();
    let members = ["env".into(), "SCHEMA".into()];
    assert_eq!(
        env::alias_env_from_chain(&facts, &file, &named("process"), &members),
        Some("SCHEMA".into())
    );
    for scope in [0, 7] {
        let local = ValueExpression::Named {
            name: "process".into(),
            scope: Some(scope),
            offset: 0,
            context: None,
        };
        assert_eq!(
            env::alias_env_from_chain(&facts, &file, &local, &members),
            None
        );
    }
    assert_eq!(
        env::alias_env_from_chain(&facts, &file, &ValueExpression::Unknown, &members),
        None
    );
    let imported = imported("process", "default", "node:process");
    let root = ValueExpression::Named {
        name: "process".into(),
        scope: Some(0),
        offset: 0,
        context: None,
    };
    assert_eq!(
        env::alias_env_from_chain(&imported, &file, &root, &members),
        Some("SCHEMA".into())
    );
}
