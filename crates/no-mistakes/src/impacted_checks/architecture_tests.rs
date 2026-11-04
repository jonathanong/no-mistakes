#[test]
fn impacted_fanout_prepares_and_builds_the_graph_once() {
    let generate = include_str!("generate.rs");
    let impacted_prepare = include_str!("generate/prepare.rs");
    let prepared = concat!(
        include_str!("../tests/prepared_plan.rs"),
        include_str!("../tests/prepared_plan_discovery.rs"),
    );
    let plan = concat!(
        include_str!("../tests/plan.rs"),
        include_str!("../tests/plan/generate.rs"),
    );

    assert_eq!(
        impacted_prepare
            .matches("PreparedTestPlanInputs::prepare(&plan_args)")
            .count(),
        1
    );
    assert_eq!(impacted_prepare.matches("inputs.finish()?").count(), 1);
    assert_eq!(generate.matches("generate_plan_with_prepared(").count(), 1);
    assert_eq!(generate.matches("generate_plan(").count(), 0);
    for repeated_prepare in [
        "collect_changed_files(",
        "discover_visible_paths(",
        "load_v2_config(",
        "resolve_tsconfig(",
        "analyze_lockfile_changes(",
    ] {
        assert_eq!(
            generate.matches(repeated_prepare).count(),
            0,
            "fanout must not call {repeated_prepare}"
        );
    }

    assert_eq!(
        prepared.matches("VisiblePathSnapshot::new(&root)").count(),
        1
    );
    assert_eq!(
        prepared
            .matches("load_v2_config_from_source_store(")
            .count(),
        1
    );
    assert_eq!(
        prepared.matches("resolve_tsconfig_from_visible(").count(),
        1
    );
    assert_eq!(
        prepared
            .matches("collect_changed_files(&args, &root)")
            .count(),
        1
    );
    assert_eq!(prepared.matches("analyze_lockfile_changes(").count(), 1);
    assert_eq!(
        prepared
            .matches("build_with_plan_files_prepared_config_and_all_facts(")
            .count(),
        1
    );
    let graph_method = prepared
        .split("pub(crate) fn graph(&self)")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn graph_is_initialized").next())
        .expect("prepared graph method");
    assert_eq!(graph_method.matches(".get_or_init(|| {").count(), 1);
    assert_eq!(
        prepared
            .matches("self.config_invalidation.get_or_init(|| {")
            .count(),
        1
    );

    let public_wrapper = plan
        .split("pub fn generate_plan(args: &PlanArgs)")
        .nth(1)
        .and_then(|source| source.split("/// Generate a framework").next())
        .expect("public test-plan wrapper");
    assert_eq!(
        public_wrapper
            .matches("PreparedTestPlanRequest::prepare(args)")
            .count(),
        1
    );
    assert_eq!(
        public_wrapper
            .matches("generate_plan_with_prepared(")
            .count(),
        1
    );
}
