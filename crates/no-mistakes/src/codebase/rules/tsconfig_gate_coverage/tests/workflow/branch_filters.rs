use super::*;

#[test]
fn ci_scanner_excludes_source_change_workflows_with_no_matching_branch() {
    let job = |project: &str| {
        format!(
            "jobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project {project}/tsconfig.json\n"
        )
    };
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                "ignored-all-branches.yml",
                &format!(
                    "on:\n  pull_request:\n    types: [synchronize]\n    branches-ignore: ['**']\n    paths: [ignored-all-branches/**]\n{}",
                    job("ignored-all-branches")
                ),
            ),
            workflow_document(
                "ordered-exclusion.yml",
                &format!(
                    "on:\n  pull_request:\n    types: [synchronize]\n    branches: [release/**, '!**']\n    paths: [ordered-exclusion/**]\n{}",
                    job("ordered-exclusion")
                ),
            ),
            workflow_document(
                "reexcluded-reintroduction.yml",
                &format!(
                    "on:\n  pull_request:\n    types: [synchronize]\n    branches: ['!**', main, '!main']\n    paths: [reexcluded-reintroduction/**]\n{}",
                    job("reexcluded-reintroduction")
                ),
            ),
            // A non-universal ignore pattern must remain a possible source-change path.
            workflow_document(
                "non-universal-ignore.yml",
                &format!(
                    "on:\n  pull_request:\n    types: [opened, synchronize]\n    branches-ignore: [release/**]\n    paths: [non-universal-ignore/**]\n{}",
                    job("non-universal-ignore")
                ),
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "ignored-all-branches/tsconfig.json".to_string(),
        "non-universal-ignore/tsconfig.json".to_string(),
        "ordered-exclusion/tsconfig.json".to_string(),
        "reexcluded-reintroduction/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from(["non-universal-ignore/tsconfig.json".to_string()])
    );
}
