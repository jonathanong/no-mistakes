use super::*;

#[test]
fn directory_project_arguments_cover_tracked_primary_tsconfig_in_ci_and_local_checks() {
    let root = fixture_root("pass");
    let report = findings(&root, &config(&root));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn workflow_paths_must_cover_every_source_selected_by_the_project() {
    let negative = fixture_root("path-filter-sources-negative");
    let report = findings(&negative, &config(&negative));
    assert_eq!(report.len(), 1, "{report:#?}");
    assert_eq!(report[0].file, "app/tsconfig.json");
    assert!(
        report[0].message.contains("no CI typecheck registration"),
        "{report:#?}"
    );

    let positive = fixture_root("path-filter-sources-positive");
    let report = findings(&positive, &config(&positive));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn reusable_workflow_callers_cover_filaments_style_static_typecheck_jobs() {
    let root = fixture_root("reusable-workflow");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 2, "{report:#?}");
    assert!(
        report
            .iter()
            .all(|finding| finding.file == "caller-only/tsconfig.json"),
        "{report:#?}"
    );
    assert!(
        report
            .iter()
            .any(|finding| finding.message.contains("no CI typecheck registration")),
        "{report:#?}"
    );
    assert!(
        report
            .iter()
            .any(|finding| finding.message.contains("no local typecheck registration")),
        "{report:#?}"
    );
}

#[test]
fn literal_array_contains_and_pull_request_activities_control_reusable_coverage() {
    let root = fixture_root("activity-conditions");
    let report = findings(&root, &config(&root));

    assert_eq!(report.len(), 2, "{report:#?}");
    assert_eq!(
        report
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["opened/tsconfig.json", "unreachable/tsconfig.json"])
    );
    assert!(report
        .iter()
        .all(|finding| finding.message.contains("no CI typecheck registration")));
}

#[test]
fn no_check_tsconfigs_do_not_credit_ci_or_local_gates() {
    let root = fixture_root("no-check");
    let report = findings(&root, &config(&root));

    assert_eq!(report.len(), 2, "{report:#?}");
    for project in ["direct/tsconfig.json", "inherited/tsconfig.json"] {
        let finding = report
            .iter()
            .find(|finding| finding.file == project)
            .unwrap_or_else(|| panic!("missing {project} finding: {report:#?}"));
        assert!(finding.message.contains("compilerOptions.noCheck is true"));
    }
    assert!(report.iter().all(|finding| !matches!(
        finding.file.as_str(),
        "override/tsconfig.json" | "invalid/tsconfig.json"
    )));
}
