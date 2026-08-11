use super::*;

#[test]
fn run_scripts_resolve_static_inputs_matrix_and_environment_values() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  input:\n    uses: ./.github/workflows/callee.yml\n    with: {project: input-project}\n",
    );
    let callee = document(
        ".github/workflows/callee.yml",
        "on:\n  workflow_call:\n    inputs:\n      project: {type: string, required: true}\njobs:\n  input:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p '${{ inputs.project }}/tsconfig.json'\n  matrix:\n    strategy:\n      matrix: {project: [matrix-project]}\n    runs-on: ubuntu-latest\n    env:\n      PROJECT: '${{ matrix.project }}'\n    steps:\n      - run: tsc --noEmit -p '${{ env.PROJECT }}/tsconfig.json'\n  dynamic:\n    needs: input\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p '${{ needs.input.outputs.project }}/tsconfig.json'\n",
    );

    assert_eq!(
        scanned_projects(
            vec![caller, callee],
            &["input-project", "matrix-project", "dynamic"],
        ),
        BTreeSet::from([
            "input-project/tsconfig.json".to_string(),
            "matrix-project/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn resolved_run_failures_block_later_steps_but_dynamic_runs_fail_open() {
    let workflow = document(
        ".github/workflows/run-resolution.yml",
        "on: push\njobs:\n  resolved:\n    env: {COMMAND: false}\n    runs-on: ubuntu-latest\n    steps:\n      - run: '${{ env.COMMAND }}'\n      - run: tsc --noEmit -p resolved/tsconfig.json\n  dynamic:\n    runs-on: ubuntu-latest\n    steps:\n      - run: '${{ github.event.head_commit.message }}'\n      - run: tsc --noEmit -p dynamic/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["resolved", "dynamic"]),
        BTreeSet::from(["dynamic/tsconfig.json".to_string()])
    );
}
