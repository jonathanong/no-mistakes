use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::rules::postgres_sql_cross_file::graph_plan;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn prepared(names: &[&str]) -> (CheckFactMap, DepGraph, PathBuf) {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID).join("issue-3073-catalogs"),
    );
    let files: Vec<_> = names
        .iter()
        .map(|name| root.join("src/coverage").join(name))
        .collect();
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-catalogs/.no-mistakes.yml"
    ))).unwrap();
    let plan = TsFactPlan {
        value_flow: true,
        source: true,
        ..TsFactPlan::imports()
    };
    let ts = collect_ts_facts(&files, plan)
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
        .collect();
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts,
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        graph_plan(&config).unwrap(),
        files,
        &facts,
    )
    .unwrap();
    (facts, graph, root.join("src/coverage"))
}

pub(super) fn binding<'a>(facts: &'a CheckFactMap, file: &Path, name: &str) -> &'a ValueBinding {
    facts
        .get_ts_facts(file)
        .unwrap()
        .value_flow
        .bindings
        .iter()
        .find(|binding| binding.name == name)
        .unwrap()
}

pub(super) fn helper(facts: &CheckFactMap, file: &Path, name: &str) -> CallableId {
    facts
        .get_ts_facts(file)
        .unwrap()
        .callable_scope_ids
        .iter()
        .find(|(_, scope)| scope == name)
        .unwrap()
        .0
}
