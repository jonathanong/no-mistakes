use super::tests::{assert_case, Case};

#[test]
fn local_actions_are_found_through_reusable_workflows_and_outside_github_actions() {
    for case in [
        Case {
            name: "reusable-workflow-action",
            roots: &[".github/workflows/ci.yml#test-tooling"],
            workflows: &[".github/workflows/ci.yml", ".github/workflows/tooling.yml"],
            global: false,
        },
        Case {
            name: "repository-action",
            roots: &[".github/workflows/ci.yml#test-web"],
            workflows: &[".github/workflows/ci.yml"],
            global: false,
        },
        Case {
            name: "root-action",
            roots: &[".github/workflows/ci.yml#test-web"],
            workflows: &[".github/workflows/ci.yml"],
            global: false,
        },
        Case {
            name: "root-action-missing",
            roots: &[
                ".github/workflows/ci.yml#test-backend",
                ".github/workflows/ci.yml#test-web",
            ],
            workflows: &[],
            global: true,
        },
    ] {
        assert_case(case);
    }
}

#[test]
fn unresolved_nested_reachable_actions_fail_open_even_when_the_parent_resolves() {
    for name in ["nested-action-missing", "nested-action-malformed"] {
        let report = super::tests::report(name);
        assert!(report.global_fallback, "{name}");
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "unowned-local-action" && diagnostic.root_job_ids.is_none()
        }));
    }
}
