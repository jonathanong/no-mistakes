use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::{ValueParameter, ValueProjection};
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use std::sync::Arc;

fn prepared() -> (CheckFactMap, DepGraph, PathBuf) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3599-origin-boundaries"),
    );
    let file = root.join("source.ts");
    let (facts, graph, _) = super::super::import_tests::prepared_files(root, vec![file.clone()]);
    (facts, graph, file)
}

fn reference(parameter: &ValueParameter) -> ValueExpression {
    ValueExpression::Named {
        name: parameter.name.clone(),
        scope: Some(parameter.scope),
        offset: u32::MAX,
        context: Some(parameter.owner),
    }
}

#[test]
fn origin_requires_prepared_source_and_known_values() {
    let (facts, graph, file) = prepared();
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let parameter = facts
        .get_ts_facts(&file)
        .unwrap()
        .value_flow
        .parameters
        .iter()
        .find(|parameter| parameter.name == "prefix")
        .unwrap();
    let mut proof = Proof::new(&graph, &opts, &facts, &bindings);
    assert!(proof.origin(&file, &reference(parameter)));
    assert!(!proof.origin(&file.with_file_name("missing.ts"), &reference(parameter)));
    let missing = ValueExpression::Named {
        name: "absent".into(),
        scope: Some(0),
        offset: u32::MAX,
        context: None,
    };
    assert!(!proof.origin(&file, &missing));
    assert!(!proof.origin(&file, &ValueExpression::Call(u32::MAX - 1, u32::MAX)));
    assert!(!proof.origin(&file, &ValueExpression::Unknown));
}

#[test]
fn origin_rejects_invalidated_and_projected_formals() {
    for projected in [false, true] {
        let (mut facts, graph, file) = prepared();
        let mut ts = facts.get_ts_facts(&file).unwrap().clone();
        let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
        {
            let bindings = Bindings::new(&facts).with_graph(&graph);
            let parameter = ts
                .value_flow
                .parameters
                .iter()
                .find(|parameter| parameter.name == "prefix")
                .unwrap();
            assert!(
                Proof::new(&graph, &opts, &facts, &bindings).origin(&file, &reference(parameter))
            );
        }
        let parameter = ts
            .value_flow
            .parameters
            .iter_mut()
            .find(|parameter| parameter.name == "prefix")
            .unwrap();
        if projected {
            parameter
                .projection
                .push(ValueProjection::Member("statement".into()));
        } else {
            parameter.invalidated_at = Some(0);
        }
        let value = reference(parameter);
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
        let bindings = Bindings::new(&facts).with_graph(&graph);
        assert!(!Proof::new(&graph, &opts, &facts, &bindings).origin(&file, &value));
    }
}

#[test]
fn origin_rejects_constructor_callers_and_missing_call_facts() {
    let (facts, graph, file) = prepared();
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let constructed = flow
        .parameters
        .iter()
        .find(|parameter| parameter.name == "constructed")
        .unwrap();
    let prefix = flow
        .parameters
        .iter()
        .find(|parameter| parameter.name == "prefix")
        .unwrap();
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let mut proof = Proof::new(&graph, &opts, &facts, &bindings);
    assert!(graph
        .resolved_call_sites()
        .iter()
        .any(|site| site.invocation
            != crate::codebase::dependencies::extract::InvocationKind::Call
            && site.targets().any(|target| matches!(target.node,
                Some(crate::codebase::dependencies::graph::NodeId::Symbol {
                    callable_id: Some(owner), ..
                }) if *owner == constructed.owner))));
    assert!(!proof.origin(&file, &reference(constructed)));
    assert!(proof.origin(&file, &reference(prefix)));
    let mut missing = Proof::new(&graph, &opts, &facts, &bindings);
    missing.file(&file).unwrap();
    missing.files.get_mut(&file).unwrap().calls_at.clear();
    assert!(!missing.origin(&file, &reference(prefix)));
}

#[test]
fn origin_projects_an_ordinary_static_member_call() {
    let (facts, graph, file) = prepared();
    let parameter = facts
        .get_ts_facts(&file)
        .unwrap()
        .value_flow
        .parameters
        .iter()
        .find(|parameter| parameter.name == "memberPrefix")
        .unwrap();
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    assert!(Proof::new(&graph, &opts, &facts, &bindings).origin(&file, &reference(parameter)));
}
