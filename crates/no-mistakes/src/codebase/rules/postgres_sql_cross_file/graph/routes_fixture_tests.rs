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
            .join("3304-registry-routes"),
    );
    let files: Vec<_> = [
        "catalog.ts",
        "barrel.ts",
        "alias.ts",
        "consumer.ts",
        "common.cjs",
        "esm-direct.ts",
        "esm-nested.ts",
        "esm-named.ts",
        "esm-alias.ts",
        "esm-absent.ts",
        "esm-not-namespace.ts",
        "cjs-direct.cjs",
        "cjs-nested.cjs",
    ]
    .into_iter()
    .map(|name| root.join("src").join(name))
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
fn registry_routes_follow_named_namespace_and_eager_require_edges_only() {
    let (facts, graph, root) = prepared();
    let routes = RegistryRoutes::new(&facts);
    let mut mismatches = Vec::new();
    for (file, local, members, expected) in [
        ("esm-direct.ts", "direct", vec!["TABLES"], true),
        ("consumer.ts", "direct", vec!["TABLES"], false),
        ("common.cjs", "direct", vec!["TABLES"], false),
        ("esm-nested.ts", "nested", vec!["group", "TABLES"], true),
        ("esm-named.ts", "group", vec!["TABLES"], true),
        ("esm-alias.ts", "aliases", vec!["renamed"], true),
        ("esm-direct.ts", "direct", vec!["missing"], false),
        (
            "esm-not-namespace.ts",
            "notNamespace",
            vec!["accounts"],
            false,
        ),
        ("esm-absent.ts", "absent", vec!["TABLES"], false),
        ("cjs-direct.cjs", "direct", vec!["TABLES"], true),
        ("cjs-nested.cjs", "nested", vec!["TABLES"], true),
        ("cjs-direct.cjs", "picked", vec![], true),
        ("cjs-direct.cjs", "alias", vec![], false),
        ("common.cjs", "dynamic", vec!["TABLES"], false),
        ("cjs-direct.cjs", "returned", vec!["TABLES"], false),
    ] {
        let path = root.join("src").join(file);
        let members: Vec<_> = members.into_iter().map(str::to_owned).collect();
        let route = super::routes::registry_route(&graph, &routes, &path, local, &members);
        if route.is_some() != expected {
            mismatches.push(format!(
                "{file}: {local}.{members:?}: expected route={expected}, got {}",
                route.is_some()
            ));
        }
        if let Some(route) = route {
            if route.producer != root.join("src/catalog.ts") || route.exported != "TABLES" {
                mismatches.push(format!(
                    "{file}: {local}.{members:?}: wrong producer/export"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
