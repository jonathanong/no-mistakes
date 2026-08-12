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

    let uncovered = report
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        uncovered,
        BTreeSet::from([
            "action-outcome/tsconfig.json",
            "branch-main-only/tsconfig.json",
            "env/tsconfig.json",
            "checkout/tsconfig.json",
            "composite/tsconfig.json",
            "composite-remote/tsconfig.json",
            "composite-secret/tsconfig.json",
            "conclusion-blocked/tsconfig.json",
            "container-dependent/tsconfig.json",
            "docker-invalid/tsconfig.json",
            "dynamic-dependent/tsconfig.json",
            "fail-fast-default/tsconfig.json",
            "fail-fast-true/tsconfig.json",
            "fork-secret/tsconfig.json",
            "hidden-ref/tsconfig.json",
            "join/tsconfig.json",
            "json-error/tsconfig.json",
            "job-status/tsconfig.json",
            "logical-and/tsconfig.json",
            "matrix/tsconfig.json",
            "output/tsconfig.json",
            "runner/tsconfig.json",
            "runner-dependent/tsconfig.json",
            "script/tsconfig.json",
            "input-condition/tsconfig.json",
            "mapping-env/tsconfig.json",
            "pre-if/tsconfig.json",
            "pr-target/tsconfig.json",
            "ref-type/tsconfig.json",
            "needs-action/tsconfig.json",
            "step-env/tsconfig.json",
            "strategy-dependent/tsconfig.json",
            "strategy-index/tsconfig.json",
            "tag-ref/tsconfig.json",
            "tags-only/tsconfig.json",
            "trigger-typo/tsconfig.json",
            "sparse-checkout/tsconfig.json",
        ]),
        "{report:#?}"
    );
    assert!(report
        .iter()
        .all(|finding| finding.message.contains("no CI typecheck registration")));

    // true || skips an invalid right operand; false && skips it and the job.
    assert!(!uncovered.contains("logical-or/tsconfig.json"));
    assert!(uncovered.contains("logical-and/tsconfig.json"));
    // A tolerated checkout still makes a following local action available.
    assert!(!uncovered.contains("checkout-tolerated/tsconfig.json"));
    // Invalid values in a step-merged environment block that step.
    assert!(uncovered.contains("step-env/tsconfig.json"));
    // Invalid runner, strategy, and container prerequisites fail their needs.
    for project in [
        "runner-dependent/tsconfig.json",
        "strategy-dependent/tsconfig.json",
        "container-dependent/tsconfig.json",
    ] {
        assert!(uncovered.contains(project));
    }
    // A tag-only push has no source-change coverage and cannot become a branch ref.
    assert!(uncovered.contains("tag-ref/tsconfig.json"));
    // Default/true fail-fast cancel later siblings; false leaves them runnable.
    assert!(uncovered.contains("fail-fast-default/tsconfig.json"));
    assert!(uncovered.contains("fail-fast-true/tsconfig.json"));
    assert!(!uncovered.contains("fail-fast-false/tsconfig.json"));
    // Runtime conditions can skip prerequisites, so ordinary needs cannot be trusted.
    assert!(uncovered.contains("dynamic-dependent/tsconfig.json"));
    // Exact branches and wildcard alternatives are separate source-change activations.
    assert!(uncovered.contains("branch-main-only/tsconfig.json"));
    // Scalar action defaults are valid, but composite secret contexts and bad Docker refs are not.
    assert!(!uncovered.contains("scalar-action/tsconfig.json"));
    assert!(uncovered.contains("composite-secret/tsconfig.json"));
    assert!(uncovered.contains("docker-invalid/tsconfig.json"));
    // Tolerated failures retain their outcome but publish a successful conclusion.
    assert!(!uncovered.contains("conclusion-outcome/tsconfig.json"));
    assert!(uncovered.contains("conclusion-blocked/tsconfig.json"));
    // A trigger typo and a potentially forked PR secret both prevent scheduling.
    assert!(uncovered.contains("trigger-typo/tsconfig.json"));
    assert!(uncovered.contains("fork-secret/tsconfig.json"));
    // Only a root checkout exposes repository-local actions.
    assert!(!uncovered.contains("checkout-root/tsconfig.json"));
    assert!(uncovered.contains("sparse-checkout/tsconfig.json"));
    // Known action, strategy, ref, and job contexts make these gates unreachable.
    for project in [
        "action-outcome/tsconfig.json",
        "strategy-index/tsconfig.json",
        "ref-type/tsconfig.json",
        "job-status/tsconfig.json",
    ] {
        assert!(uncovered.contains(project));
    }
    // Invalid composite targets, tag-only pushes, and base-only PR targets cannot cover changes.
    for project in [
        "composite-remote/tsconfig.json",
        "tags-only/tsconfig.json",
        "pr-target/tsconfig.json",
    ] {
        assert!(uncovered.contains(project));
    }
    // pnpm permits an optional command separator before a static typecheck.
    assert!(!uncovered.contains("pnpm-separator/tsconfig.json"));
}

#[test]
fn exact_branch_activations_must_all_cover_a_project() {
    let root = fixture_root("reusable-branch-intersection");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 1, "{report:#?}");
    assert_eq!(report[0].file, "app/tsconfig.json");
    assert!(report[0].message.contains("no CI typecheck registration"));
}

#[test]
fn nonexistent_static_working_directories_stop_later_typechecks() {
    let root = fixture_root("nonexistent-working-directory");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 1, "{report:#?}");
    assert_eq!(report[0].file, "app/tsconfig.json");
    assert!(report[0].message.contains("no CI typecheck registration"));
}

#[test]
fn reusable_static_outputs_activate_downstream_needs_conditions() {
    let root = fixture_root("reusable-static-outputs");
    let report = findings(&root, &config(&root));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn tolerated_failures_are_effective_needs_successes() {
    let root = fixture_root("tolerated-failure-needs");
    let report = findings(&root, &config(&root));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn unsupported_case_functions_do_not_credit_typechecks() {
    let root = fixture_root("unsupported-case-expression");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 1, "{report:#?}");
    assert_eq!(report[0].file, "app/tsconfig.json");
    assert!(report[0].message.contains("no CI typecheck registration"));
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
