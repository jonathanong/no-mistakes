use super::*;

#[test]
fn resolved_fail_fast_values_must_be_boolean() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {fail_fast: '{}'}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {fail_fast: 'true'}\n",
        ),
        workflow(
            ".github/workflows/invalid.yml",
            "on:\n  workflow_call:\n    inputs:\n      fail_fast: {type: string, required: true}\njobs:\n  typecheck:\n    strategy:\n      fail-fast: '${{ fromJSON(inputs.fail_fast) }}'\n      matrix: {target: [one]}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-fail-fast/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid.yml",
            "on:\n  workflow_call:\n    inputs:\n      fail_fast: {type: string, required: true}\njobs:\n  typecheck:\n    strategy:\n      fail-fast: '${{ fromJSON(inputs.fail_fast) }}'\n      matrix: {target: [one]}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-fail-fast/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/matrix.yml",
            "on: push\njobs:\n  invalid:\n    strategy:\n      fail-fast: '${{ fromJSON(matrix.fail_fast) }}'\n      matrix:\n        fail_fast: ['[]']\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-matrix-fail-fast/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "invalid-fail-fast/tsconfig.json",
        "valid-fail-fast/tsconfig.json",
        "invalid-matrix-fail-fast/tsconfig.json",
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
        BTreeSet::from(["valid-fail-fast/tsconfig.json".to_string()])
    );
}
