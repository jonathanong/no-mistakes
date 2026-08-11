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
    assert_eq!(report.len(), 4, "{report:#?}");
    assert!(
        report.iter().all(|finding| matches!(
            finding.file.as_str(),
            "caller-only/tsconfig.json"
                | "invalid-literal-fail-fast/tsconfig.json"
                | "invalid-literal-matrix/tsconfig.json"
        )),
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
    for project in [
        "invalid-literal-fail-fast/tsconfig.json",
        "invalid-literal-matrix/tsconfig.json",
    ] {
        assert!(report.iter().any(|finding| {
            finding.file == project && finding.message.contains("no CI typecheck registration")
        }));
        assert!(report.iter().all(|finding| {
            finding.file != project || !finding.message.contains("no local typecheck registration")
        }));
    }
    assert!(report
        .iter()
        .all(|finding| finding.file != "literal-closing-braces/tsconfig.json"));
}

#[test]
fn local_action_steps_require_existing_valid_action_metadata() {
    let root = fixture_root("local-action-targets");
    let report = findings(&root, &config(&root));

    assert_eq!(
        report
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "failing-action/tsconfig.json",
            "invalid/tsconfig.json",
            "missing-dockerfile/tsconfig.json",
            "missing-entrypoint/tsconfig.json",
            "missing-working-directory/tsconfig.json",
            "missing/tsconfig.json",
            "malformed-metadata/tsconfig.json",
        ]),
        "{report:#?}"
    );
    assert!(report
        .iter()
        .all(|finding| finding.message.contains("no CI typecheck registration")));
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
fn reusable_activation_state_controls_secret_env_and_event_defaults() {
    let root = fixture_root("reusable-activation-state");
    let report = findings(&root, &config(&root));

    assert_eq!(report.len(), 4, "{report:#?}");
    for project in [
        "omitted-job/tsconfig.json",
        "omitted-step/tsconfig.json",
        "push-default/tsconfig.json",
        "schedule-default/tsconfig.json",
    ] {
        let finding = report
            .iter()
            .find(|finding| finding.file == project)
            .unwrap_or_else(|| panic!("missing {project} finding: {report:#?}"));
        assert!(finding.message.contains("no CI typecheck registration"));
    }
}

#[test]
fn reusable_workflow_review_regressions_do_not_credit_unrunnable_typechecks() {
    let root = fixture_root("reusable-review-regressions");
    let report = findings(&root, &config(&root));

    assert_eq!(
        report
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "env/tsconfig.json",
            "checkout/tsconfig.json",
            "composite/tsconfig.json",
            "hidden-ref/tsconfig.json",
            "join/tsconfig.json",
            "json-error/tsconfig.json",
            "matrix/tsconfig.json",
            "output/tsconfig.json",
            "runner/tsconfig.json",
            "script/tsconfig.json",
        ]),
        "{report:#?}"
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
