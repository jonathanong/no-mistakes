use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

fn prepared() -> (CheckFactMap, PathBuf) {
    let file = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-text-origins/src/callers.ts"),
    );
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let files = vec![file.clone(), file.with_file_name("external.ts")];
    let collected = collect_ts_facts(&files, plan);
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts: files
            .into_iter()
            .map(|file| {
                let ts = collected.get(&file).unwrap();
                assert!(ts.parse_error.is_none(), "{:?}", ts.parse_error);
                (
                    file,
                    CheckFileFacts {
                        ts: Arc::new(ts.clone()),
                        ..Default::default()
                    }
                    .into(),
                )
            })
            .collect(),
        ..Default::default()
    };
    (facts, file)
}

fn tags() -> [TrustedSqlTag; 1] {
    [TrustedSqlTag {
        module: "@example/sql".into(),
        name: "sql".into(),
    }]
}

#[test]
fn caller_origins_preserve_nullable_awaited_and_projected_scalar_values() {
    let (facts, file) = prepared();
    let root = file.parent().unwrap();
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        root,
        &TsConfig {
            dir: root.into(),
            paths_dir: root.into(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        facts.files.clone(),
        &facts,
    )
    .unwrap();
    assert!(super::super::caller_index(&graph, &file, &Default::default()).is_empty());
    let tags = tags();
    let bindings = Bindings::new(&facts)
        .with_graph(&graph)
        .with_trusted_tags(&tags);
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    for (name, expected) in [
        ("nullableCallerSlot", true),
        ("awaitedCallerSlot", true),
        ("nullishCallerSlot", true),
        ("memberCallerSlot", true),
        ("arrayMemberCallerSlot", true),
        ("containerCallerSlot", false),
        ("undefinedBindSlot", true),
        ("awaitedBindSlot", true),
        ("nullishBindSlot", true),
        ("defaultProjectionSlot", false),
    ] {
        let binding = flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let ValueExpression::Call(start, end) = binding.value else {
            panic!("tag {name}");
        };
        let tag = flow
            .calls
            .iter()
            .find(|call| (call.offset, call.end) == (start, end))
            .unwrap();
        assert_eq!(
            is_text_tag(&bindings, &file, tag, &tags),
            expected,
            "{name}"
        );
    }
}

#[test]
fn alternative_origins_require_both_scalar_branches_and_their_literal_receipts() {
    let facts = CheckFactMap::default();
    let bindings = Bindings::new(&facts);
    let file = Path::new("missing.ts");
    let literal = TextOrigin {
        literal: true,
        ..Default::default()
    };
    let origin = TextOrigin {
        parts: vec![Some(literal.clone()), Some(literal)],
        ..Default::default()
    };
    let unknown = Arc::new(ValueExpression::Unknown);
    let null = Arc::new(ValueExpression::Null);
    let undefined = Arc::new(ValueExpression::Undefined);
    for value in [
        ValueExpression::Either(unknown.clone(), unknown.clone()),
        ValueExpression::Default(unknown.clone(), unknown),
        ValueExpression::Nullish(null.clone(), undefined.clone()),
    ] {
        assert!(ordinary(
            &bindings,
            file,
            &value,
            Some(&origin),
            &[],
            &[],
            &mut ProofState::default()
        ));
    }
    let unsafe_value = ValueExpression::Either(null, Arc::new(ValueExpression::Unknown));
    assert!(!ordinary(
        &bindings,
        file,
        &unsafe_value,
        None,
        &[],
        &[],
        &mut ProofState::default()
    ));
    for value in [
        ValueExpression::Null,
        ValueExpression::Undefined,
        ValueExpression::Awaited(undefined),
    ] {
        assert!(arguments::bound_argument(
            &bindings,
            file,
            &value,
            &[],
            &[],
            &mut ProofState::default()
        ));
        assert!(!arguments::bound_argument(
            &bindings,
            file,
            &value,
            &["field".into()],
            &[],
            &mut ProofState::default()
        ));
    }
    for value in [
        ValueExpression::Either(
            Arc::new(ValueExpression::Null),
            Arc::new(ValueExpression::Undefined),
        ),
        ValueExpression::Default(
            Arc::new(ValueExpression::Null),
            Arc::new(ValueExpression::Null),
        ),
        ValueExpression::Nullish(
            Arc::new(ValueExpression::Undefined),
            Arc::new(ValueExpression::Null),
        ),
    ] {
        assert!(arguments::bound_argument(
            &bindings,
            file,
            &value,
            &[],
            &[],
            &mut ProofState::default()
        ));
    }
}

#[test]
fn missing_prepared_facts_cannot_prove_named_arguments_or_tagged_text() {
    let (prepared, file) = prepared();
    let flow = &prepared.get_ts_facts(&file).unwrap().value_flow;
    let tag = flow
        .calls
        .iter()
        .find(|call| flow.tagged_templates.contains_key(&(call.offset, call.end)))
        .unwrap();
    let facts = CheckFactMap::default();
    let bindings = Bindings::new(&facts);
    let value = ValueExpression::Named {
        name: "value".into(),
        scope: Some(1),
        offset: 0,
        context: None,
    };
    assert!(!ordinary(
        &bindings,
        &file,
        &value,
        None,
        &[],
        &[],
        &mut ProofState::default()
    ));
    assert!(!arguments::bound_argument(
        &bindings,
        &file,
        &value,
        &[],
        &[],
        &mut ProofState::default()
    ));
    assert!(!is_text_tag(&bindings, &file, tag, &tags()));
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        file.parent().unwrap().into(),
        Default::default(),
        Default::default(),
    );
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let mut parameter = flow
        .parameters
        .iter()
        .find(|parameter| parameter.name == "value")
        .unwrap()
        .clone();
    parameter.default = ValueExpression::Unknown;
    parameter.projection.clear();
    assert!(!super::super::parameter_bound(
        &bindings,
        &file,
        &parameter,
        0,
        &[],
        &[],
        &mut ProofState::default()
    ));
}
