use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn canonical_helper_calls_prove_only_guarded_static_metadata_fields() {
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
