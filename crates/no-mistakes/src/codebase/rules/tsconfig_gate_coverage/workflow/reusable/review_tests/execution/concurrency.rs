use super::*;

#[test]
fn max_queued_concurrency_preserves_reusable_workflow_coverage() {
    let documents = vec![
        document(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  typecheck:\n    uses: ./.github/workflows/callee.yml\n",
        ),
        document(
            ".github/workflows/callee.yml",
            "on: workflow_call\nconcurrency:\n  group: workflow-checks\n  queue: max\njobs:\n  typecheck:\n    concurrency:\n      group: job-checks\n      queue: max\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p queued-concurrency/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned_projects(documents, &["queued-concurrency"]),
        BTreeSet::from(["queued-concurrency/tsconfig.json".to_string()])
    );
}

#[test]
fn resolved_workflow_concurrency_groups_gate_reusable_activations() {
    let documents = vec![
        document(
            ".github/workflows/invalid-caller.yml",
            "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {group: ''}\n",
        ),
        document(
            ".github/workflows/valid-caller.yml",
            "on: push\njobs:\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {group: checks}\n",
        ),
        document(
            ".github/workflows/embedded-caller.yml",
            "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/embedded.yml\n    with: {group: '{}'}\n",
        ),
        document(
            ".github/workflows/invalid.yml",
            "on:\n  workflow_call:\n    inputs:\n      group: {type: string, required: true}\nconcurrency: '${{ inputs.group }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p reusable-concurrency/tsconfig.json\n",
        ),
        document(
            ".github/workflows/valid.yml",
            "on:\n  workflow_call:\n    inputs:\n      group: {type: string, required: true}\nconcurrency: '${{ inputs.group }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p reusable-concurrency/tsconfig.json\n",
        ),
        document(
            ".github/workflows/embedded.yml",
            "on:\n  workflow_call:\n    inputs:\n      group: {type: string, required: true}\nconcurrency: 'checks-${{ fromJSON(inputs.group) }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p embedded-concurrency/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned_projects(documents, &["reusable-concurrency", "embedded-concurrency"],),
        BTreeSet::from(["reusable-concurrency/tsconfig.json".to_string()])
    );
}

#[test]
fn resolved_matrix_concurrency_groups_gate_job_instances() {
    let workflow = document(
        ".github/workflows/matrix-concurrency.yml",
        "on: push\njobs:\n  typecheck:\n    strategy:\n      # This test isolates per-instance concurrency validation from fail-fast cancellation.\n      fail-fast: false\n      matrix:\n        include:\n          - project: invalid-matrix-concurrency\n            group: ''\n          - project: valid-matrix-concurrency\n            group: checks\n    concurrency: '${{ matrix.group }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p '${{ matrix.project }}/tsconfig.json'\n  blocked:\n    needs: typecheck\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked-after-concurrency/tsconfig.json\n  continues:\n    needs: typecheck\n    if: always() && needs.typecheck.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-after-concurrency/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "invalid-matrix-concurrency",
                "valid-matrix-concurrency",
                "blocked-after-concurrency",
                "failure-after-concurrency",
            ],
        ),
        BTreeSet::from([
            "failure-after-concurrency/tsconfig.json".to_string(),
            "valid-matrix-concurrency/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn invalid_reusable_job_concurrency_propagates_failure_to_needs() {
    let documents = vec![
        document(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  call:\n    strategy:\n      matrix: {group: ['']}\n    concurrency: '${{ matrix.group }}'\n    uses: ./.github/workflows/callee.yml\n  blocked:\n    needs: call\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked-after-call/tsconfig.json\n  continues:\n    needs: call\n    if: always() && needs.call.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-after-call/tsconfig.json\n",
        ),
        document(
            ".github/workflows/callee.yml",
            "on: workflow_call\njobs:\n  noop:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n",
        ),
    ];

    assert_eq!(
        scanned_projects(documents, &["blocked-after-call", "failure-after-call"],),
        BTreeSet::from(["failure-after-call/tsconfig.json".to_string()])
    );
}
