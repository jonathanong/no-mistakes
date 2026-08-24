use super::tests::{assert_case, Case};

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
fn needs_closure_does_not_pull_siblings_through_a_prerequisite() {
    assert_case(Case {
        name: "needs-sibling",
        roots: &[
            ".github/workflows/ci.yml#prepare",
            ".github/workflows/ci.yml#test-web",
        ],
        workflows: &[".github/workflows/ci.yml"],
        global: false,
    });
}

#[test]
fn needs_closure_includes_prerequisites_of_affected_dependents() {
    assert_case(Case {
        name: "needs-dependent-prerequisite",
        roots: &[
            ".github/workflows/ci.yml#publish",
            ".github/workflows/ci.yml#release-notes",
            ".github/workflows/ci.yml#test-web",
        ],
        workflows: &[".github/workflows/ci.yml"],
        global: false,
    });
}
