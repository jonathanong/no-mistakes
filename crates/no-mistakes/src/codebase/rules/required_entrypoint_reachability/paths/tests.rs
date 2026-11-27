use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};

#[test]
fn selected_malformed_source_is_not_reported_unreachable() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation"),
    );
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let broken = root.join("broken.ts");
    let clean = root.join("clean.ts");
    let error = require_sources(&graph, [&broken]).unwrap_err();
    assert!(error.to_string().contains("broken.ts"), "{error:#}");
    assert!(require_sources(&graph, [&clean]).is_ok());
}

#[test]
fn entrypoints_with_empty_normalized_root_reject_parent_escapes() {
    let root = Path::new(".");
    assert!(resolve_entrypoint(root, "../outside.ts").is_none());
    assert_eq!(
        resolve_entrypoint(root, "worker/src/index.mts"),
        Some(PathBuf::from("worker/src/index.mts"))
    );
}
