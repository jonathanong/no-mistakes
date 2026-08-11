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
