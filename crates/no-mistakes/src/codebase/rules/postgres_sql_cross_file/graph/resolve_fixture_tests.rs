use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

fn prepared() -> (CheckFactMap, DepGraph, PathBuf) {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("3304-crossfile-resolve"),
    );
    let files: Vec<_> = [
        "src/consumer.ts",
        "src/barrel.ts",
        "src/catalog.ts",
        "src/ambiguous-consumer.ts",
        "src/ambiguous.ts",
        "src/other.ts",
    ]
    .iter()
    .map(|file| root.join(file))
    .collect();
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts: collect_ts_facts(&files, plan)
            .into_iter()
            .map(|(file, ts)| {
                assert!(ts.parse_error.is_none(), "{file:?}: {:?}", ts.parse_error);
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
            imports: true,
            value_import_selectors: true,
            ..Default::default()
        },
        files,
        &facts,
    )
    .unwrap();
    (facts, graph, root)
}

#[test]
fn saved_imports_resolve_unique_aliases_and_reject_ambiguous_star_exports() {
    let (facts, graph, root) = prepared();
    let consumer = root.join("src/consumer.ts");
    let unique = unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &consumer,
        "registry",
    )
    .unwrap();
    assert_eq!(unique.producer, root.join("src/catalog.ts"));
    assert_eq!(unique.exported, "TABLE_NAMES");

    let table = unique_type_export(&graph, &facts, &consumer, "TableName").unwrap();
    assert_eq!(table.producer, root.join("src/catalog.ts"));
    assert_eq!(table.exported, "TableName");

    let ambiguous_consumer = root.join("src/ambiguous-consumer.ts");
    let candidates = possible_exports(&graph, &facts, &ambiguous_consumer, "TABLE_NAMES");
    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates
            .iter()
            .map(|item| item.producer.clone())
            .collect::<Vec<_>>(),
        [root.join("src/catalog.ts"), root.join("src/other.ts")]
    );
    assert!(unique_export(
        &graph,
        &facts,
        &RegistryRoutes::new(&facts),
        &ambiguous_consumer,
        "TABLE_NAMES"
    )
    .is_none());
}
