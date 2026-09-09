use super::*;

#[test]
fn nested_aggregate_callable_members_reach_only_invoked_bodies_and_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/nested-callable-aggregates/fixture"),
    );
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let source = root.join("src/aggregate-callables.mts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&source),
        TsFactPlan {
            function_calls: true,
            imports: true,
            ..TsFactPlan::default()
        },
    );
    let file_facts = facts
        .get(&source)
        .expect("fixture source must produce TS facts");
    let reachable = reachable_function_scopes(file_facts);
    for scope in ["boot/registry/load", "boot/Service/run", "boot/Service/reload"] {
        assert!(
            file_facts
                .callable_scope_ids
                .iter()
                .any(|(id, candidate)| candidate == scope && reachable.contains(id)),
            "{scope} must be reached from its invoked aggregate member: {:#?}",
            file_facts.function_calls
        );
    }
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .expect("aggregate callable graph must build");
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let imports = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<HashSet<_>>();

    for path in [
        "object-called.mts",
        "field-called.mts",
        "field-reloaded.mts",
    ] {
        assert!(
            imports.contains(root.join("src").join(path).as_path()),
            "invoked aggregate member must retain {path}: {imports:?}"
        );
    }
    for path in ["object-unused.mts", "field-unused.mts"] {
        assert!(
            !imports.contains(root.join("src").join(path).as_path()),
            "uninvoked aggregate member must remain pruned: {path}"
        );
    }
}
