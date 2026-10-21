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
