use super::*;

#[test]
fn failing_local_actions_block_needs_dependents_from_credit() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  invalid-action:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./.github/actions/missing\n  blocked:\n    needs: invalid-action\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked/tsconfig.json\n  independent:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p independent/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "blocked/tsconfig.json".to_string(),
        "independent/tsconfig.json".to_string(),
    ]);
    let tracked_paths = tracked
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();

    assert_eq!(
        collect_ci_projects_with_local_actions(
            std::path::Path::new("."),
            &workflows,
            &tracked,
            &tracked_paths,
            &project_inputs(&tracked),
            &BTreeSet::new(),
        )
        .0,
        BTreeSet::from(["independent/tsconfig.json".to_string()])
    );
}

#[test]
fn local_reusable_workflow_failures_propagate_to_callers() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n  dependent:\n    needs: call\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n  always:\n    needs: call\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p caller-always/tsconfig.json\n  failure-handler:\n    needs: call\n    if: failure()\n    uses: ./.github/workflows/gate.yml\n  after-handler:\n    needs: failure-handler\n    if: always() && needs.failure-handler.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p reusable-after-handler/tsconfig.json\n",
    );
    let callee = document(
        ".github/workflows/callee.yml",
        "on: workflow_call\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n",
    );
    let gate = document(
        ".github/workflows/gate.yml",
        "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-handler/tsconfig.json\n      - run: exit 1\n",
    );

    assert_eq!(
        scanned_projects(
            vec![caller, callee, gate],
            &[
                "dependent",
                "caller-always",
                "failure-handler",
                "reusable-after-handler",
            ],
        ),
        BTreeSet::from([
            "caller-always/tsconfig.json".to_string(),
            "reusable-after-handler/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn successful_failure_handlers_are_known_not_skipped() {
    let direct = document(
        ".github/workflows/direct-handler.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n  handler:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'true'\n  skipped-result:\n    needs: handler\n    if: always() && needs.handler.result == 'skipped'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p skipped-result/tsconfig.json\n  not-skipped-result:\n    needs: handler\n    if: always() && needs.handler.result != 'skipped'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p not-skipped-result/tsconfig.json\n",
    );
    let caller = document(
        ".github/workflows/reusable-handler.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n  handler:\n    needs: setup\n    if: failure()\n    uses: ./.github/workflows/success.yml\n  skipped-result:\n    needs: handler\n    if: always() && needs.handler.result == 'skipped'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p reusable-skipped-result/tsconfig.json\n  not-skipped-result:\n    needs: handler\n    if: always() && needs.handler.result != 'skipped'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p reusable-not-skipped-result/tsconfig.json\n",
    );
    let success = document(
        ".github/workflows/success.yml",
        "on: workflow_call\njobs:\n  success:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'true'\n",
    );

    assert_eq!(
        scanned_projects(
            vec![direct, caller, success],
            &[
                "skipped-result",
                "not-skipped-result",
                "reusable-skipped-result",
                "reusable-not-skipped-result",
            ],
        ),
        BTreeSet::from([
            "not-skipped-result/tsconfig.json".to_string(),
            "reusable-not-skipped-result/tsconfig.json".to_string(),
        ])
    );
}
