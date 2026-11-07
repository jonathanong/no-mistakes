use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan, TsFileFacts};
use std::path::PathBuf;
use std::sync::Arc;

fn prepared() -> (PathBuf, CheckFactMap, DepGraph) {
    let root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards",
    ));
    let file = root.join("src/metadata-helpers.ts");
    let plan = TsFactPlan {
        value_flow: true,
        source: true,
        ..TsFactPlan::imports()
    };
    let prepared = collect_ts_facts(std::slice::from_ref(&file), plan);
    let ts = prepared.get(&file).unwrap();
    assert!(ts.parse_error.is_none());
    let mut facts = CheckFactMap {
        files: vec![file.clone()],
        graph_plan: plan,
        ..Default::default()
    };
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts.clone()),
            ..Default::default()
        }
        .into(),
    );
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        vec![file.clone()],
        &facts,
    )
    .unwrap();
    (file, facts, graph)
}

fn replace(facts: &mut CheckFactMap, file: &Path, ts: TsFileFacts) {
    facts.ts.insert(
        file.to_path_buf(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
}

#[test]
fn canonical_helper_calls_prove_only_guarded_static_metadata_fields() {
    let (file, facts, graph) = prepared();
    let ts = facts.get_ts_facts(&file).unwrap();
    for (name, expected, count) in [
        ("getTableOrThrow", true, 3),
        ("unchecked", false, 1),
        ("withFallback", false, 1),
        ("open", false, 1),
        ("asyncLookup", false, 1),
        ("objectLookup", false, 1),
        ("taintedLookup", false, 1),
        ("rawLookup", false, 1),
    ] {
        let calls: Vec<_> = ts
            .value_flow
            .calls
            .iter()
            .filter(|call| call.source_callee == name)
            .collect();
        assert_eq!(calls.len(), count, "{name}");
        for call in calls {
            assert_eq!(
                helper_call(&graph, &facts, &file, &call.callee),
                expected,
                "{name}"
            );
            assert_eq!(
                helper_call(
                    &graph,
                    &facts,
                    &file,
                    &ValueExpression::Call(call.offset, call.end)
                ),
                expected,
                "{name} binding"
            );
        }
    }
}

#[test]
fn metadata_proof_boundaries_require_complete_producers_and_unique_return_bindings() {
    let (file, mut facts, graph) = prepared();
    let baseline = facts.get_ts_facts(&file).unwrap().clone();
    let call = baseline
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "getTableOrThrow")
        .unwrap();
    let value = ValueExpression::Call(call.offset, call.end);
    assert!(helper_call(&graph, &facts, &file, &value));
    facts.ts.remove(&file);
    assert!(!helper_call(&graph, &facts, &file, &value));
    let owner = baseline
        .callable_scope_ids
        .iter()
        .find(|(_, name)| name == "getTableOrThrow")
        .unwrap()
        .0;
    let binding = baseline
        .value_flow
        .bindings
        .iter()
        .find(|binding| binding.owner == Some(owner) && binding.name == "selected")
        .unwrap();
    for failure in [
        "parse",
        "incomplete",
        "missing binding",
        "duplicate binding",
    ] {
        let mut ts = baseline.clone();
        match failure {
            "parse" => ts.parse_error = Some("missing producer syntax".into()),
            "incomplete" => ts.value_flow.incomplete_reason = Some("analysis budget".into()),
            "missing binding" => ts
                .value_flow
                .bindings
                .retain(|candidate| candidate != binding),
            "duplicate binding" => ts.value_flow.bindings.push(binding.clone()),
            _ => unreachable!(),
        }
        replace(&mut facts, &file, ts);
        assert!(!helper_call(&graph, &facts, &file, &value), "{failure}");
    }
}

#[test]
fn metadata_proof_boundaries_reject_writes_between_guard_and_return() {
    let (file, mut facts, graph) = prepared();
    let baseline = facts.get_ts_facts(&file).unwrap().clone();
    let call = baseline
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "getTableOrThrow")
        .unwrap();
    let value = ValueExpression::Call(call.offset, call.end);
    let owner = baseline
        .callable_scope_ids
        .iter()
        .find(|(_, name)| name == "getTableOrThrow")
        .unwrap()
        .0;
    let guard = baseline
        .value_flow
        .finite_identifier_guards
        .iter()
        .find(|guard| guard.context == Some(owner) && guard.name == "selected")
        .unwrap();
    let GuardSubject::Binding { scope, .. } = guard.subject.unwrap() else {
        panic!("binding guard")
    };
    let offset = baseline
        .value_flow
        .returns
        .iter()
        .find_map(|(id, value)| {
            let ValueExpression::Member(base, _) = value else {
                return None;
            };
            match base.as_ref() {
                ValueExpression::Named { offset, .. } if *id == owner => Some(*offset),
                _ => None,
            }
        })
        .unwrap();
    for (at, expected) in [(guard.from, true), (guard.from + 1, false), (offset, true)] {
        let mut ts = baseline.clone();
        let mut write = baseline
            .value_flow
            .identifier_writes
            .first()
            .unwrap()
            .clone();
        write.scope = scope;
        write.name = "selected".into();
        write.members = vec![Some("table".into())];
        write.at = at;
        ts.value_flow.identifier_writes.push(write);
        replace(&mut facts, &file, ts);
        assert_eq!(helper_call(&graph, &facts, &file, &value), expected, "{at}");
    }
}
