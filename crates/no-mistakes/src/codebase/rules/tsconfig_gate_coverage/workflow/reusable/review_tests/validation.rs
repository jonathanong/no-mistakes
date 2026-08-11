use super::*;

#[test]
fn unavailable_reusable_secret_expressions_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    secrets:\n      token: '${{ success() }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-secret/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["invalid-secret/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn invalid_continue_on_error_and_environment_contexts_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/job-continue.yml",
                "on: push\njobs:\n  typecheck:\n    continue-on-error: '${{ failure() }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-job-continue/tsconfig.json\n",
            ),
            document(
                ".github/workflows/step-continue.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: '${{ failure() }}'\n        run: tsc --noEmit -p invalid-step-continue/tsconfig.json\n",
            ),
            document(
                ".github/workflows/environment.yml",
                "on: push\njobs:\n  typecheck:\n    environment: '${{ secrets.DEPLOY_ENV }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-environment/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on: push\njobs:\n  typecheck:\n    environment:\n      name: '${{ matrix.deployment }}'\n      url: '${{ steps.deploy.outputs.url }}'\n    strategy:\n      matrix:\n        deployment: [staging]\n    runs-on: ubuntu-latest\n    steps:\n      - id: deploy\n        run: tsc --noEmit -p valid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-job-continue/tsconfig.json".to_string(),
        "invalid-step-continue/tsconfig.json".to_string(),
        "invalid-environment/tsconfig.json".to_string(),
        "valid/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}
