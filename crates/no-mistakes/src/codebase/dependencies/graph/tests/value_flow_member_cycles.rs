use super::*;

#[test]
fn cyclic_member_initializers_terminate_without_losing_acyclic_targets() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-member-cycles"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let cycles = graph.call_sites_in_file(&root.join("src/cycles.mts"));
    assert_eq!(cycles.len(), 4, "all cyclic calls must remain represented");
    assert!(
        cycles
            .iter()
            .all(|site| matches!(site.target, ResolvedCallTarget::Unknown)),
        "cyclic field projections cannot manufacture a Known target: {cycles:#?}"
    );
    let acyclic = graph.call_sites_in_file(&root.join("src/acyclic.mts"));
    assert_eq!(acyclic.len(), 4);
    assert!(
        acyclic.iter().all(|site| matches!(&site.target,
        ResolvedCallTarget::RepositoryFunction { scope, .. } if scope == "target")),
        "completed field reads must not poison repeated aliases or finite chains: {acyclic:#?}"
    );
    let recovery = graph.call_sites_in_file(&root.join("src/recovery.mts"));
    assert_eq!(recovery.len(), 2);
    assert!(matches!(recovery[0].target, ResolvedCallTarget::Unknown));
    assert!(
        matches!(&recovery[1].target,
        ResolvedCallTarget::RepositoryFunction { scope, .. } if scope == "survived"),
        "a cycle must not poison a later independent projection: {recovery:#?}"
    );
}
