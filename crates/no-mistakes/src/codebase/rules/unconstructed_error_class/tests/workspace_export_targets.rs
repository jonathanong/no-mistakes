use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, NodeId};
use crate::codebase::ts_resolver::TsConfig;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

fn root() -> PathBuf {
    fixture()
        .parent()
        .unwrap()
        .join("workspace-export-targets/fixture")
}

#[test]
fn workspace_export_constructor_metadata_retains_concrete_class_identity() {
    let root = root();
    let graph = DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            extends: true,
            ..Default::default()
        },
    )
    .unwrap();
    for (callee, path, name) in [
        (
            "DynamicConfig",
            "packages/cache-core/dynamic-config.mts",
            "DynamicConfig",
        ),
        ("Filter", "packages/cache-core/filter.mts", "Filter"),
        ("SelectedError", "app/queues/errors.mts", "UsedError"),
    ] {
        let site = graph
            .resolved_call_sites()
            .iter()
            .find(|site| {
                site.file.ends_with("app/queues/config.mts") && site.source_callee == callee
            })
            .unwrap();
        let target = graph
            .constructor_class(&site.file, site.offset)
            .expect("workspace import names alone do not prove constructor identity");
        let NodeId::Symbol {
            file,
            symbol,
            callable_id: Some(_),
        } = target
        else {
            panic!("{target:?}")
        };
        assert!(file.ends_with(path), "{target:?}");
        assert_eq!(symbol.as_ref(), name, "{target:?}");
    }
}
