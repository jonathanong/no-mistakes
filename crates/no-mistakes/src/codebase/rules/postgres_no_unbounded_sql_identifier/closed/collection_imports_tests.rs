use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

fn prepared() -> (CheckFactMap, DepGraph, PathBuf) {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-imported-guards"),
    );
    let files: Vec<_> = [
        "tables.ts",
        "db.ts",
        "alias-readonly.ts",
        "alias-namespace-export.ts",
        "alias-namespace-export-tables.ts",
        "alias-cycle-a.ts",
        "alias-cycle-b.ts",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect();
    let plan = TsFactPlan {
        value_flow: true,
        function_calls: true,
        ..TsFactPlan::imports()
    };
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts: collect_ts_facts(&files, plan)
            .into_iter()
            .map(|(file, ts)| {
                assert!(ts.parse_error.is_none(), "{}", file.display());
                (
                    file,
                    CheckFileFacts {
                        ts: Arc::new(ts),
                        ..Default::default()
                    }
                    .into(),
                )
            })
            .collect(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        files,
        &facts,
    )
    .unwrap();
    (facts, graph, root)
}

#[test]
fn imported_collection_proof_requires_complete_reachable_facts() {
    let (mut facts, graph, root) = prepared();
    let producer = UniqueExport {
        producer: root.join("tables.ts"),
        exported: "ALLOWED_TABLES".into(),
    };
    assert!(safe(&graph, &facts, &Bindings::new(&facts), &producer));
    assert!(!safe(
        &graph,
        &CheckFactMap::default(),
        &Bindings::new(&facts),
        &producer
    ));
    for (file, expected) in [("alias-cycle-a.ts", true), ("alias-readonly.ts", false)] {
        let path = root.join(file);
        let original = facts.ts.remove(&path).unwrap();
        let mut ts = original.ts.as_ref().clone();
        ts.value_flow.incomplete_reason = Some("incomplete dependency facts".into());
        facts.ts.insert(
            path.clone(),
            CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(
            safe(&graph, &facts, &Bindings::new(&facts), &producer),
            expected,
            "{file}"
        );
        facts.ts.insert(path, original);
    }
    let escaped = UniqueExport {
        producer: root.join("alias-namespace-export-tables.ts"),
        exported: "EXPORTED_TABLES".into(),
    };
    assert!(!safe(&graph, &facts, &Bindings::new(&facts), &escaped));
    let bindings = Bindings::new(&facts);
    assert!(!super::super::collections::proven(
        &graph,
        &facts,
        &bindings,
        &root.join("missing.ts"),
        "ALLOWED_TABLES",
        RegistryKind::Array
    ));
    assert!(!super::super::collections::proven(
        &graph,
        &facts,
        &bindings,
        &root.join("alias-readonly.ts"),
        "missing",
        RegistryKind::Array
    ));
}

#[test]
fn import_reachability_terminates_cycles_and_distinguishes_other_producers() {
    let (_, graph, root) = prepared();
    assert!(reaches(
        &graph,
        &root.join("alias-cycle-a.ts"),
        &root.join("alias-cycle-b.ts")
    ));
    assert!(!reaches(
        &graph,
        &root.join("alias-cycle-a.ts"),
        &root.join("tables.ts")
    ));
    assert!(!reaches(
        &graph,
        &root.join("alias-readonly.ts"),
        &root.join("alias-cycle-a.ts")
    ));
}
