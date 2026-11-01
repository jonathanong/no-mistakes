use super::ProductionBases;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
use crate::codebase::ts_resolver::TsConfig;
use std::path::PathBuf;

#[test]
fn non_extends_edges_stay_beside_a_production_base() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/production-exports-only-used-by-tests/fixture/class-extends-production",
        ),
    );
    let graph = DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..TsConfig::default()
        },
        GraphBuildPlan {
            extends: true,
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let class = |scope: &str| {
        graph
            .class_declarations()
            .iter()
            .find(|class| class.scope == scope)
            .unwrap_or_else(|| panic!("{scope}"))
    };
    assert!(graph
        .dependencies_of_node(&class("GlobalChild").node())
        .is_none());
    let dependencies = graph.dependencies_of_node(&class("Sub").node()).unwrap();
    assert!(dependencies
        .iter()
        .any(|(_, kind)| *kind == EdgeKind::Extends));
    assert!(dependencies
        .iter()
        .any(|(_, kind)| *kind != EdgeKind::Extends));
    let bases = ProductionBases::collect(&graph, |path| path.starts_with(root.join("src")));
    let (target, _) = dependencies
        .iter()
        .find(|(_, kind)| *kind == EdgeKind::Extends)
        .unwrap();
    assert!(matches!(
        target,
        NodeId::Symbol {
            file,
            callable_id: Some(id),
            ..
        } if bases.ids.contains(&(file.as_ref().to_path_buf(), *id))
    ));
}
