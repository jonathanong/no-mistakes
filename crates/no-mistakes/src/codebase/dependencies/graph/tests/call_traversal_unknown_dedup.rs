use super::*;

#[test]
fn synthetic_callback_offsets_do_not_hide_real_unknown_calls() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("call-traversal"));
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
            imports: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let unknown = root.join("src/unknown-offset.mts");

    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == unknown
            && site.source_callee == "<unknown>"
            && site.offset == 0
            && site.invocation == InvocationKind::Call
            && matches!(site.target, ResolvedCallTarget::Unknown)
    }));
}
