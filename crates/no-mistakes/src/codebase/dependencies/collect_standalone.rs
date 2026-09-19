pub(crate) fn collect_and_filter_entries(
    args: &TraverseArgs,
    direction: Direction,
    cwd_early: &Path,
    timings: &mut crate::codebase::timing::PhaseTimings,
) -> Result<TraversalResult> {
    let root = resolve_root(args, cwd_early);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let allowed = relationship_filter(&args.relationships);
    let build_plan = graph::GraphBuildPlan::from_allowed(allowed.as_ref())
        .with_symbols(traversal_needs_symbol_facts(args));
    let mut framework_plan =
        crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(build_plan);
    framework_plan.include_framework_names(args.tests.iter().map(String::as_str));
    let shared = SharedTraversalContext::prepare_with_framework_plan_for_direction(
        root,
        args.tsconfig.as_deref(),
        None,
        build_plan,
        framework_plan,
        direction,
    );
    let mut shared = shared?;
    validate_candidate_bounds(args, direction)?;
    if args.has_candidate_bounds() {
        shared.apply_candidate_inventory(args)?;
    }

    timings.mark("search");
    timings.mark("ingest");
    let result = collect_and_filter_entries_shared(args, direction, cwd_early, &mut shared)?;
    timings.mark("parse");
    timings.mark("analysis");
    Ok(result)
}
