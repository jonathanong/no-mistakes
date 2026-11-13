use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, TsFactLookup};
use crate::codebase::rules::postgres_sql_cross_file::{Bindings, UniqueExport};
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Deserialize)]
struct Case {
    name: String,
    files: Vec<String>,
    producer: String,
    expected: bool,
}

fn prepared(root: &Path, case: &Case) -> (CheckFactMap, DepGraph) {
    let files: Vec<PathBuf> = case.files.iter().map(|name| root.join(name)).collect();
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
                if ts.parse_error.is_some() {
                    assert!(file
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("broken-"));
                }
                if file.ends_with("broken-related.ts") {
                    assert!(ts.parse_error.is_some());
                    assert!(!ts.fatal_parse_error);
                    assert_eq!(ts.imports.len(), 1);
                }
                if file.ends_with("broken-unrelated.ts") {
                    assert!(ts.parse_error.is_some());
                    assert!(ts.fatal_parse_error);
                }
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
        root,
        &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
        GraphBuildPlan {
            imports: true,
            calls: true,
            ..Default::default()
        },
        files,
        &facts,
    )
    .unwrap();
    (facts, graph)
}

#[test]
fn collection_import_safety_requires_closed_reads_without_reachable_escapes() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("3304-collection-imports"),
    );
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(root.join("cases.json")).unwrap()).unwrap();
    let expected: BTreeMap<_, _> = cases
        .iter()
        .map(|case| (case.name.clone(), case.expected))
        .collect();
    let actual = cases
        .iter()
        .map(|case| {
            let (facts, graph) = prepared(&root, case);
            let producer = UniqueExport {
                producer: root.join(&case.producer),
                exported: "TABLES".into(),
            };
            assert!(facts
                .get_ts_facts(&producer.producer)
                .unwrap()
                .exported_bindings
                .iter()
                .any(|binding| binding.exported == producer.exported));
            let valid = super::safe(&graph, &facts, &Bindings::new(&facts), &producer);
            (case.name.clone(), valid)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected);
}
