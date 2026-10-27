use super::*;
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

fn findings(case: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    run_check(
        &malformed_root(),
        Some(&super::forbidden_calls_case(case)),
        None,
    )
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
    let error = findings("malformed-file-root").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("src/broken.mts"), "{message}");
    assert!(message.contains("failed to parse"), "{message}");
    assert!(!message.contains("no callable source"), "{message}");
}

#[test]
fn malformed_function_root_fails_closed_before_empty_expansion() {
    let error = findings("malformed-function-root").unwrap_err();
    assert!(error.to_string().contains("failed to parse"), "{error:#}");
}

#[test]
fn malformed_vitest_root_file_fails_closed() {
    let error = findings("vitest-named-unit").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("src/unit/broken.test.mts"), "{message}");
    assert!(message.contains("failed to parse"), "{message}");
}
