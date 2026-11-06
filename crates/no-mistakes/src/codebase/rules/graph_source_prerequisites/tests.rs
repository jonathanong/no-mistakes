use super::*;
use std::path::PathBuf;

#[test]
fn required_import_failure_does_not_poison_unrelated_roots() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan {
            imports: true,
            calls: true,
            mock_boundaries: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(require_import_sources(
        "consumer",
        &graph,
        &[NodeId::file(root.join("mocked.ts"))],
        None
    )
    .is_ok());
    assert!(require_import_sources(
        "consumer",
        &graph,
        &[NodeId::file(root.join("type-only.ts"))],
        None
    )
    .is_ok());
    let error = require_import_sources(
        "consumer",
        &graph,
        &[NodeId::file(root.join("entry.ts"))],
        None,
    )
    .unwrap_err();
    assert!(error.to_string().contains("broken.ts"), "{error:#}");
    assert!(require_import_sources(
        "consumer",
        &graph,
        &[NodeId::file(root.join("clean.ts"))],
        None
    )
    .is_ok());
}

#[test]
fn healthy_sources_require_no_runtime_import_walk() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/runtime-import-sharing"),
    );
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        crate::codebase::dependencies::graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    require_import_sources("consumer", &graph, &[NodeId::file(root.join("a.ts"))], None).unwrap();
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("graph.runtime_import_visits")
            .copied()
            .unwrap_or(0),
        0
    );
}
