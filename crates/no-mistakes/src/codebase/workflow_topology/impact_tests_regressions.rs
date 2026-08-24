use super::super::model::DiagnosticCode;
use super::diagnostics::requires_unbound_global;
use super::tests::{assert_case, report, Case};

#[test]
fn needs_closure_unions_both_revisions_and_both_directions() {
    assert_case(Case {
        name: "needs-union",
        roots: &[
            ".github/workflows/ci.yml#prepare",
            ".github/workflows/ci.yml#publish",
            ".github/workflows/ci.yml#test-web",
        ],
        workflows: &[".github/workflows/ci.yml"],
        global: false,
    });
}

#[test]
fn composite_action_changes_resolve_direct_and_nested_callers() {
    for (name, roots) in [
        ("direct-action", &[".github/workflows/ci.yml#test-web"][..]),
        (
            "nested-action",
            &[".github/workflows/ci.yml#test-tooling"][..],
        ),
        ("action-added", &[".github/workflows/ci.yml#test-web"][..]),
        ("action-deleted", &[".github/workflows/ci.yml#test-web"][..]),
        ("action-renamed", &[".github/workflows/ci.yml#test-web"][..]),
    ] {
        assert_case(Case {
            name,
            roots,
            workflows: &[".github/workflows/ci.yml"],
            global: false,
        });
    }
}

#[test]
fn unowned_action_fails_open_with_a_global_diagnostic() {
    let report = report("unowned-action");
    assert!(report.global_fallback);
    assert_eq!(report.affected_root_job_ids.len(), 2);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "unowned-local-action" && diagnostic.root_job_ids.is_none()
    }));
}

#[test]
fn changed_action_does_not_report_non_entry_workflow_users() {
    assert_case(Case {
        name: "action-outside-entry",
        roots: &[".github/workflows/ci.yml#test-web"],
        workflows: &[".github/workflows/ci.yml"],
        global: false,
    });
}

#[test]
fn reachable_reusable_name_ambiguity_is_global() {
    let report = report("reachable-name-ambiguity");
    assert!(report.global_fallback);
}

#[test]
fn missing_or_ambiguous_unrepresented_endpoints_are_global_without_root_evidence() {
    for (name, code) in [
        ("missing-local-workflow", "missing-local-workflow"),
        ("missing-workflow-run-source", "missing-workflow-run-source"),
        (
            "ambiguous-workflow-run-source",
            "ambiguous-workflow-run-source",
        ),
    ] {
        let report = report(name);
        assert!(report.global_fallback, "{name}");
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == code && diagnostic.root_job_ids.is_none() }));
    }
}

#[test]
fn global_only_diagnostic_codes_cannot_be_localized_from_partial_endpoints() {
    for code in [
        DiagnosticCode::DuplicateWorkflowName,
        DiagnosticCode::MissingNeedsDependency,
        DiagnosticCode::MissingLocalWorkflow,
        DiagnosticCode::MissingWorkflowRunSource,
        DiagnosticCode::AmbiguousWorkflowRunSource,
        DiagnosticCode::WorkflowRunCycle,
        DiagnosticCode::WorkflowRunChainLimit,
        DiagnosticCode::MissingArtifactProducer,
        DiagnosticCode::AmbiguousArtifactProducer,
        DiagnosticCode::ArtifactResolutionLimit,
    ] {
        assert!(requires_unbound_global(code), "{}", code.as_str());
    }
}

#[test]
fn workflow_run_cycle_across_separate_entry_roots_is_global_without_root_evidence() {
    let report = report("workflow-run-cycle");
    assert!(report.global_fallback);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "workflow-run-cycle" && diagnostic.root_job_ids.is_none()
    }));
}

#[test]
fn entry_jobs_mapping_addition_fails_open_instead_of_returning_empty() {
    assert_case(Case {
        name: "entry-jobs-added",
        roots: &[
            ".github/workflows/ci.yml#test-backend",
            ".github/workflows/ci.yml#test-web",
        ],
        workflows: &[".github/workflows/ci.yml"],
        global: true,
    });
}

#[test]
fn deleted_or_non_mapping_entry_jobs_fail_open_to_base_roots() {
    for name in ["entry-jobs-deleted", "entry-jobs-nonmapping"] {
        assert_case(Case {
            name,
            roots: &[".github/workflows/ci.yml#test-web"],
            workflows: &[".github/workflows/ci.yml"],
            global: true,
        });
    }
}
