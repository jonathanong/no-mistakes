use super::*;

#[test]
fn tolerated_checkout_only_makes_local_actions_available_after_valid_action_inputs() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/checks.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        continue-on-error: true\n      - uses: ./.github/actions/setup\n      - run: tsc --noEmit -p app/tsconfig.json\n",
            ),
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/invalid-checkout.yml\n    with: {payload: '{}'}\n",
            ),
            document(
                ".github/workflows/invalid-checkout.yml",
                "on:\n  workflow_call:\n    inputs:\n      payload: {type: string, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        continue-on-error: true\n        with: {ref: '${{ fromJSON(inputs.payload) }}'}\n      - uses: ./.github/actions/setup\n      - run: tsc --noEmit -p invalid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "app/tsconfig.json".to_string(),
        "invalid/tsconfig.json".to_string(),
    ]);
    let local_actions = BTreeSet::from([".github/actions/setup".to_string()]);

    assert_eq!(
        collect_ci_projects_with_local_actions(
            &workflows,
            &tracked,
            &project_inputs(&tracked),
            &local_actions,
        )
        .0,
        BTreeSet::from(["app/tsconfig.json".to_string()])
    );
}
