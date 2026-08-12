use super::*;

#[test]
fn tolerated_checkout_only_makes_local_actions_available_after_valid_action_inputs() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/checks.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        continue-on-error: true\n        with: {path: .}\n      - uses: ./.github/actions/setup\n      - run: tsc --noEmit -p app/tsconfig.json\n",
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
    let tracked_paths = tracked
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let local_actions = super::super::super::super::local_actions::LocalActionCatalog::non_docker(
        BTreeSet::from([".github/actions/setup".to_string()]),
    );

    assert_eq!(
        collect_ci_projects_with_local_actions(
            std::path::Path::new("."),
            &workflows,
            &tracked,
            &tracked_paths,
            &project_inputs(&tracked),
            &local_actions,
        )
        .0,
        BTreeSet::from(["app/tsconfig.json".to_string()])
    );
}

#[test]
fn local_docker_actions_require_a_statically_linux_runner() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/docker-actions.yml",
            "on: push\njobs:\n  windows:\n    runs-on: windows-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./.github/actions/docker\n      - shell: bash\n        run: tsc --noEmit -p windows/tsconfig.json\n  linux:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./.github/actions/docker\n      - run: tsc --noEmit -p linux/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "windows/tsconfig.json".to_string(),
        "linux/tsconfig.json".to_string(),
    ]);
    let tracked_paths = tracked
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let local_actions =
        super::super::super::super::local_actions::LocalActionCatalog::docker(BTreeSet::from([
            ".github/actions/docker".to_string(),
        ]));

    assert_eq!(
        collect_ci_projects_with_local_actions(
            std::path::Path::new("."),
            &workflows,
            &tracked,
            &tracked_paths,
            &project_inputs(&tracked),
            &local_actions,
        )
        .0,
        BTreeSet::from(["linux/tsconfig.json".to_string()])
    );
}
