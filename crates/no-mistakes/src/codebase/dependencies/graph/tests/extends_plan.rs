use super::*;

fn extends_only() -> GraphBuildPlan {
    GraphBuildPlan {
        extends: true,
        ..GraphBuildPlan::default()
    }
}

#[test]
fn extends_is_opt_in_and_never_part_of_the_full_plan() {
    assert!(!GraphBuildPlan::all().extends);
    assert!(!GraphBuildPlan::from_allowed(None).extends);
    assert!(!GraphBuildPlan::default().extends);
}

#[test]
fn allowed_extends_requests_the_hierarchy_and_the_imports_it_resolves_through() {
    let allowed: HashSet<_> = [EdgeKind::Extends].into();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.extends && plan.imports);
    // Extends alone never turns `Call` edges on.
    assert!(!plan.calls);

    let calls: HashSet<_> = [EdgeKind::Call].into();
    assert!(!GraphBuildPlan::from_allowed(Some(&calls)).extends);
}

#[test]
fn including_a_plan_unions_extends_without_touching_calls() {
    let mut plan = GraphBuildPlan {
        calls: true,
        ..GraphBuildPlan::default()
    };
    plan.include(GraphBuildPlan::default());
    assert!(plan.calls && !plan.extends);
    plan.include(extends_only());
    assert!(plan.calls && plan.extends);

    let mut only = extends_only();
    only.include(GraphBuildPlan::default());
    assert!(only.extends && !only.calls);
}

#[test]
fn extends_needs_the_function_call_facts_class_bases_are_recorded_in() {
    assert!(extends_only().ts_fact_plan().function_calls);
    assert!(!GraphBuildPlan::default().ts_fact_plan().function_calls);
}

#[test]
fn extends_without_prepared_facts_is_refused() {
    assert_eq!(
        require_core_edge_facts(extends_only(), None)
            .expect_err("extends edges cannot build without prepared facts")
            .to_string(),
        "TS call facts are required when extends edges are requested"
    );
}

/// The default relationship set is call-pruned, so a walk over it must never
/// follow a class base, while an unfiltered walk sees every kind present.
#[test]
fn default_traversal_excludes_extends_edges() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("class-bases"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(&root, &tsconfig, extends_only()).unwrap();
    let local = graph
        .class_declarations()
        .iter()
        .find(|class| class.scope == "Local")
        .unwrap()
        .node();
    let via = |allowed: Option<&HashSet<EdgeKind>>| -> Vec<Vec<EdgeKind>> {
        graph
            .deps_of(std::slice::from_ref(&local), None, allowed)
            .into_iter()
            .filter(|entry| entry.node != local)
            .map(|entry| entry.via)
            .collect()
    };

    let standard = crate::codebase::dependencies::relationship_filter(&[]).unwrap();
    assert!(!standard.contains(&EdgeKind::Extends));
    assert!(via(Some(&standard)).is_empty());
    assert_eq!(via(None), [vec![EdgeKind::Extends]]);
    let requested: HashSet<_> = [EdgeKind::Extends].into();
    assert_eq!(via(Some(&requested)), [vec![EdgeKind::Extends]]);
}
