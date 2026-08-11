use super::*;

#[test]
fn non_stringable_action_inputs_do_not_credit_later_typechecks() {
    let documents = vec![
        workflow(
            ".github/workflows/mapping.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: \"${{ fromJSON('{}') }}\"\n      - run: tsc --noEmit --project mapping/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/sequence.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: \"${{ fromJSON('[]') }}\"\n      - run: tsc --noEmit --project sequence/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/scalar.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: \"${{ fromJSON('\\\"main\\\"') }}\"\n      - run: tsc --noEmit --project scalar/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/embedded.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: \"prefix-${{ fromJSON('{}') }}\"\n      - run: tsc --noEmit --project embedded/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/reusable-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/reusable-callee.yml\n    with: {payload: '{}'}\n",
        ),
        workflow(
            ".github/workflows/reusable-callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      payload: {type: string, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: '${{ fromJSON(inputs.payload) }}'\n      - run: tsc --noEmit --project reusable/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/matrix.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        payload: ['[]']\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: '${{ fromJSON(matrix.payload) }}'\n      - run: tsc --noEmit --project matrix/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "mapping/tsconfig.json",
        "sequence/tsconfig.json",
        "scalar/tsconfig.json",
        "embedded/tsconfig.json",
        "reusable/tsconfig.json",
        "matrix/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(
            &ParsedWorkflowSet { documents },
            &tracked,
            &project_inputs(&tracked),
        ),
        BTreeSet::from(["scalar/tsconfig.json".to_string()])
    );
}

#[test]
fn repository_root_local_actions_allow_following_typechecks() {
    let documents = ParsedWorkflowSet {
        documents: vec![workflow(
            ".github/workflows/root-action.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./\n      - run: tsc --noEmit --project root-action/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from(["root-action/tsconfig.json".to_string()]);

    assert_eq!(
        super::super::super::workflow::ci_typechecked_projects_with_local_actions_and_stats(
            &documents,
            &tracked,
            &project_inputs(&tracked),
            &BTreeSet::from([String::new()]),
        )
        .0,
        tracked
    );
}

#[test]
fn local_actions_require_a_prior_checkout_in_each_job() {
    let documents = ParsedWorkflowSet {
        documents: vec![
            workflow(
                ".github/workflows/direct-missing.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project direct-missing/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/direct-checked-out.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project direct-checked-out/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/other-repository.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with: {repository: other/repository}\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project other-repository/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/subdirectory.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with: {path: source}\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project subdirectory/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/invalid-checkout-input.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with: {fetch-depth: \"${{ fromJSON('not-json') }}\"}\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project invalid-checkout-input/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n  call-checked-out:\n    uses: ./.github/workflows/callee-checked-out.yml\n",
            ),
            workflow(
                ".github/workflows/callee.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project reusable-missing/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/callee-checked-out.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./.github/actions/local\n      - run: tsc --noEmit --project reusable-checked-out/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "direct-checked-out/tsconfig.json".to_string(),
        "direct-missing/tsconfig.json".to_string(),
        "invalid-checkout-input/tsconfig.json".to_string(),
        "other-repository/tsconfig.json".to_string(),
        "reusable-checked-out/tsconfig.json".to_string(),
        "reusable-missing/tsconfig.json".to_string(),
        "subdirectory/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        super::super::super::workflow::ci_typechecked_projects_with_local_actions_and_stats(
            &documents,
            &tracked,
            &project_inputs(&tracked),
            &BTreeSet::from([".github/actions/local".to_string()]),
        )
        .0,
        BTreeSet::from([
            "direct-checked-out/tsconfig.json".to_string(),
            "reusable-checked-out/tsconfig.json".to_string(),
        ])
    );
}
