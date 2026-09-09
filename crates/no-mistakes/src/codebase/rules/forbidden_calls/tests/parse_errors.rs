use super::*;
use crate::napi_api::options::test_json_arg;
use serde_json::json;
use std::path::PathBuf;

fn malformed_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/malformed-roots/fixture"),
    )
}

fn malformed_graph() -> (PathBuf, crate::codebase::dependencies::graph::DepGraph) {
    let root = malformed_root();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn findings(yaml: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = malformed_root();
    let config = tempfile::Builder::new().suffix(".yml").tempfile()?;
    std::fs::write(config.path(), yaml)?;
    run_check(&root, Some(config.path()), None)
}

#[test]
fn expand_fails_closed_for_a_malformed_file_root() {
    let (root, graph) = malformed_graph();
    let error = super::expand_roots(
        &root,
        &options("roots: [{ file: src/broken.mts }]\ntargets: [{ global: setTimeout }]"),
        &graph,
        &[],
    )
    .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("src/broken.mts"), "{message}");
    assert!(message.contains("failed to parse"), "{message}");
    assert!(!message.contains("no callable source"), "{message}");
}

#[test]
fn expand_accepts_a_valid_root_when_unrelated_files_are_malformed() {
    let (root, graph) = malformed_graph();
    let nodes = super::expand_roots(
        &root,
        &options("roots: [{ file: src/entry.mts }]\ntargets: [{ global: setTimeout }]"),
        &graph,
        &[],
    )
    .expect("unrelated parse errors must stay optional");
    assert!(!nodes.is_empty());
}

#[test]
fn malformed_file_root_fails_closed_with_a_parse_diagnostic() {
    let error = findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/broken.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("src/broken.mts"), "{message}");
    assert!(message.contains("failed to parse"), "{message}");
    assert!(!message.contains("no callable source"), "{message}");
}

#[test]
fn unrelated_malformed_files_do_not_fail_a_valid_root() {
    let found = findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/entry.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].file, "src/entry.mts");
}

#[test]
fn malformed_function_root_fails_closed_before_empty_expansion() {
    let error = findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/broken.mts, symbol: broken } }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap_err();
    assert!(error.to_string().contains("failed to parse"), "{error:#}");
}

#[test]
fn malformed_vitest_root_file_fails_closed() {
    let error = findings(
        "tests:\n  vitest:\n    configs: vitest.config.ts\nrules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ vitest: [unit] }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("src/unit/broken.test.mts"), "{message}");
    assert!(message.contains("failed to parse"), "{message}");
}

#[test]
fn check_json_surfaces_a_malformed_configured_root() {
    let root = malformed_root();
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/broken.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let output = crate::napi_api::check_json_impl(test_json_arg(
        json!({
            "root": root,
            "config": config.path()
        })
        .to_string(),
    ))
    .expect("aggregate check reports parse errors as warnings");
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(
        value["warnings"].as_array().unwrap().iter().any(|warning| {
            warning.as_str().is_some_and(|warning| {
                warning.contains("failed to parse") && warning.contains("src/broken.mts")
            })
        }),
        "{value}"
    );
    assert_eq!(value["rules"].as_array().map(Vec::len), Some(0));
}
