use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

struct Prepared {
    root: PathBuf,
    file: PathBuf,
    facts: CheckFactMap,
    graph: DepGraph,
}

fn prepared(name: &str) -> Prepared {
    let root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards",
    ));
    let file = root.join("src").join(name);
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
    Prepared {
        root,
        file,
        facts,
        graph,
    }
}

#[test]
fn row_field_proofs_require_the_canonical_closed_helper_and_support_scalar_aliases() {
    let Prepared {
        root,
        file,
        facts,
        graph,
    } = prepared("metadata-row-helpers.ts");
    let ts = facts.get_ts_facts(&file).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let calls: Vec<_> = ts
        .value_flow
        .calls
        .iter()
        .filter(|call| call.source_callee == "getRowOrThrow")
        .collect();
    assert_eq!(calls.len(), 5);
    for call in calls {
        let value = ValueExpression::Call(call.offset, call.end);
        assert!(projection(
            &graph,
            &facts,
            &bindings,
            &file,
            &value,
            &["table".into()]
        ));
        assert!(projection(
            &graph,
            &facts,
            &bindings,
            &file,
            &ValueExpression::Member(Arc::new(value.clone()), "schema".into()),
            &[]
        ));
        assert!(!projection(
            &graph,
            &facts,
            &bindings,
            &file,
            &value,
            &["missing".into()]
        ));
    }
    let aliases: Vec<_> = ts
        .value_flow
        .bindings
        .iter()
        .filter(|binding| binding.name == "table")
        .collect();
    assert_eq!(aliases.len(), 2);
    for alias in aliases {
        assert!(projection(
            &graph,
            &facts,
            &bindings,
            &file,
            &alias.value,
            &[]
        ));
    }
    let empty_graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root,
        Default::default(),
        Default::default(),
    );
    let call = ts
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "getRowOrThrow")
        .unwrap();
    assert!(!projection(
        &empty_graph,
        &facts,
        &bindings,
        &file,
        &ValueExpression::Call(call.offset, call.end),
        &["table".into()]
    ));
}

#[test]
fn same_named_parameter_calls_do_not_inherit_the_module_helper_proof() {
    let Prepared {
        file, facts, graph, ..
    } = prepared("metadata-row-shadow.ts");
    let ts = facts.get_ts_facts(&file).unwrap();
    assert_eq!(ts.value_flow.metadata_row_helpers.len(), 1);
    let call = ts
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "scopedRow")
        .unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    assert!(!projection(
        &graph,
        &facts,
        &bindings,
        &file,
        &ValueExpression::Call(call.offset, call.end),
        &["table".into()]
    ));
}

#[test]
fn metadata_proof_boundaries_require_complete_same_file_row_summaries() {
    let Prepared {
        file,
        mut facts,
        graph,
        ..
    } = prepared("metadata-row-helpers.ts");
    let baseline = facts.get_ts_facts(&file).unwrap().clone();
    let call = baseline
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "getRowOrThrow")
        .unwrap();
    let value = ValueExpression::Call(call.offset, call.end);
    assert!(projection(
        &graph,
        &facts,
        &Bindings::new(&facts),
        &file,
        &value,
        &["table".into()]
    ));
    facts.ts.remove(&file);
    assert!(!projection(
        &graph,
        &facts,
        &Bindings::new(&facts),
        &file,
        &value,
        &["table".into()]
    ));
    for failure in ["parse", "operational", "incomplete"] {
        let mut ts = baseline.clone();
        match failure {
            "parse" => ts.parse_error = Some("missing row syntax".into()),
            "operational" => ts.operational_error = Some("source unavailable".into()),
            "incomplete" => ts.value_flow.incomplete_reason = Some("analysis budget".into()),
            _ => unreachable!(),
        }
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
        assert!(
            !projection(
                &graph,
                &facts,
                &Bindings::new(&facts),
                &file,
                &value,
                &["table".into()]
            ),
            "{failure}"
        );
    }
}
