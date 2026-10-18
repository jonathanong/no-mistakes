use super::*;

#[test]
fn aggregate_vitest_ci_coverage_reuses_the_request_snapshot() {
    let prepared = concat!(
        include_str!("../prepared.rs"),
        include_str!("../../codebase/ci_workflows/prepared.rs"),
    );
    let tasks = check_task_sources();
    let dispatcher = concat!(
        include_str!("../../codebase/rules/filesystem_dispatch.rs"),
        include_str!("../../codebase/rules/filesystem_dispatch/execute.rs"),
        include_str!("../../codebase/rules/filesystem_dispatch/execute/special.rs"),
    );
    let catalog = include_str!("../../codebase/rules/vitest_project_catalog.rs");
    let mapping = include_str!("../../codebase/rules/vitest_project_mapping/project_sources.rs");
    let coverage = include_str!("../../codebase/rules/vitest_ci_path_coverage/projects.rs");
    let workflows =
        include_str!("../../codebase/rules/vitest_ci_path_coverage/workflow_filters.rs");

    assert_eq!(
        prepared.matches("prepare_vitest_project_catalog(").count(),
        1
    );
    assert_eq!(
        prepared
            .matches("prepare_playwright_project_catalog(")
            .count(),
        1
    );
    assert!(tasks.contains("run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts"));
    assert!(dispatcher.contains("check_with_files_and_catalog"));
    assert!(dispatcher.contains("check_with_files_from_snapshot_catalog_sources_and_workflows"));
    assert_eq!(
        prepared
            .matches("ParsedWorkflowSet::load_from_snapshot_and_sources(")
            .count(),
        1
    );
    assert!(prepared.contains("TSCONFIG_GATE_COVERAGE"));
    assert_eq!(
        catalog
            .matches("load_projects_from_visible_with_catalog(")
            .count(),
        1
    );
    assert!(!mapping.contains("VisiblePathSnapshot::new"));
    assert!(!coverage.contains("VisiblePathSnapshot::new"));
    let aggregate_mapping = mapping
        .split("if let Some(catalog) = catalog {")
        .nth(1)
        .and_then(|source| source.split("let projects = if").next())
        .expect("mapping prepared-catalog branch");
    assert!(
        aggregate_mapping.contains("catalog.merged_projects()"),
        "mapping must reuse the prepared catalog merge"
    );
    assert!(!aggregate_mapping.contains("load_projects("));
    let aggregate_coverage = coverage
        .split("Some(catalog) => catalog.config_projects()?")
        .nth(1)
        .and_then(|source| source.split("None =>").next())
        .expect("coverage prepared-catalog branch");
    assert!(!aggregate_coverage.contains("load_projects("));
    assert!(workflows.contains("discover_workflow_files_from_snapshot"));
    assert!(!workflows.contains("discover_workflow_files(root"));
}
