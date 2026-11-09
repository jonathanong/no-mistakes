use super::*;
use crate::react_traits::analyze::file::analyze_file;

#[test]
fn empty_component_proof_preserves_complete_file_analysis_and_skips_detail_work() {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            FULL_COMPONENT_ANALYSIS.with(|value| value.set(false));
        }
    }
    let saved = crate::test_support::materialize_saved_fixture(&fixture(
        "react-traits-components",
        "empty-proof",
    ));
    let root = saved.path();
    for (name, components) in [
        ("service.mts", 0),
        ("helper.mts", 0),
        ("Upper.tsx", 2),
        ("Anonymous.tsx", 1),
        ("Class.tsx", 1),
        ("reexport.tsx", 0),
        ("local-reexport.tsx", 1),
        ("jsx-without-component.tsx", 0),
        ("Parent.tsx", 1),
        ("Child.tsx", 1),
    ] {
        let run = |full| {
            FULL_COMPONENT_ANALYSIS.with(|value| value.set(full));
            let _restore = Restore;
            let observer = crate::diagnostics::InvocationObserver::new(true);
            let result = crate::diagnostics::with_observer(Some(observer.clone()), || {
                analyze_file(&root.join("src").join(name), root).unwrap()
            });
            let passes = observer
                .snapshot()
                .work
                .get("react.detail_passes")
                .copied()
                .unwrap_or(0);
            (
                serde_json::to_value(result.components.as_ref()).unwrap(),
                passes,
            )
        };
        let (before, old_passes) = run(true);
        let (after, new_passes) = run(false);
        assert_eq!(after, before, "{name}");
        assert_eq!(after.as_array().unwrap().len(), components, "{name}");
        assert_eq!(old_passes, 1, "{name}");
        assert_eq!(new_passes, u64::from(components != 0), "{name}");
    }
}

#[test]
fn default_forbidden_dependency_graph_retains_render_edges_and_skips_empty_react_analysis() {
    use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, NodeId};
    use crate::codebase::dependencies::EdgeKind;
    let saved = crate::test_support::materialize_saved_fixture(&fixture(
        "react-traits-components",
        "empty-proof",
    ));
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let plan = crate::codebase::rules::forbidden_dependencies::try_graph_plan(&config)
        .unwrap()
        .unwrap();
    assert_eq!(plan, GraphBuildPlan::all());
    assert!(plan.react);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let result = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root":root}),
        ))
        .unwrap()
    });
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    let findings = result["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{result:#}");
    assert_eq!(findings[0]["rule"], "forbidden-dependencies");
    assert_eq!(findings[0]["file"], "src/Parent.tsx");
    assert_eq!(
        observer.snapshot().work.get("react.detail_passes"),
        Some(&6)
    );
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig(None, &root).unwrap();
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            imports: true,
            react: true,
            ..Default::default()
        },
    )
    .unwrap();
    let allowed = std::collections::HashSet::from([EdgeKind::ReactRender]);
    let children = graph.deps_of(
        &[NodeId::file(root.join("src/Parent.tsx"))],
        None,
        Some(&allowed),
    );
    assert!(
        children.iter().any(
            |entry| entry.node == NodeId::file(root.join("src/Child.tsx"))
                && entry.via.contains(&EdgeKind::ReactRender)
        ),
        "{children:#?}"
    );
}
