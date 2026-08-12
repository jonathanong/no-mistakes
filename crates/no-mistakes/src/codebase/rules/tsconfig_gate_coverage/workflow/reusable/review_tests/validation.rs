use super::*;

mod checkout;

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
fn unknown_trigger_names_invalidate_an_otherwise_reachable_workflow() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/typo.yml",
            "on: {push: null, pussh: {}}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      # GitHub rejects the typo instead of scheduling the push job.\n      - run: tsc --noEmit -p typo/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from(["typo/tsconfig.json".to_string()]);

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

#[test]
fn resolved_step_continue_on_error_must_be_boolean() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  invalid-object:\n    uses: ./.github/workflows/object.yml\n    with: {payload: '{}'}\n  invalid-array:\n    uses: ./.github/workflows/array.yml\n    with: {payload: '[]'}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {payload: 'false'}\n",
            ),
            document(
                ".github/workflows/object.yml",
                "on:\n  workflow_call:\n    inputs:\n      payload: {type: string, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: '${{ fromJSON(inputs.payload) }}'\n        run: echo setup\n      - run: tsc --noEmit -p object/tsconfig.json\n",
            ),
            document(
                ".github/workflows/array.yml",
                "on:\n  workflow_call:\n    inputs:\n      payload: {type: string, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: '${{ fromJSON(inputs.payload) }}'\n        run: echo setup\n      - run: tsc --noEmit -p array/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on:\n  workflow_call:\n    inputs:\n      payload: {type: string, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: '${{ fromJSON(inputs.payload) }}'\n        run: tsc --noEmit -p valid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "array/tsconfig.json".to_string(),
        "object/tsconfig.json".to_string(),
        "valid/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}

#[test]
fn local_actions_are_validated_only_when_their_step_executes() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p sibling/tsconfig.json\n  skipped-job:\n    if: false\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/missing\n  missing-before:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/missing\n      - run: tsc --noEmit -p before/tsconfig.json\n  missing-after:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p after/tsconfig.json\n      - uses: ./.github/actions/missing\n  skipped-step:\n    runs-on: ubuntu-latest\n    steps:\n      - if: false\n        uses: ./.github/actions/missing\n      - run: tsc --noEmit -p skipped-step/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "after/tsconfig.json".to_string(),
        "before/tsconfig.json".to_string(),
        "sibling/tsconfig.json".to_string(),
        "skipped-step/tsconfig.json".to_string(),
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
            &super::super::super::local_actions::LocalActionCatalog::default(),
        )
        .0,
        BTreeSet::from([
            "after/tsconfig.json".to_string(),
            "sibling/tsconfig.json".to_string(),
            "skipped-step/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn resolved_job_timeouts_must_be_positive() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {timeout: 0}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {timeout: 1}\n",
            ),
            document(
                ".github/workflows/invalid.yml",
                "on:\n  workflow_call:\n    inputs:\n      timeout: {type: number, required: true}\njobs:\n  typecheck:\n    timeout-minutes: '${{ inputs.timeout }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-timeout/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on:\n  workflow_call:\n    inputs:\n      timeout: {type: number, required: true}\njobs:\n  typecheck:\n    timeout-minutes: '${{ inputs.timeout }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid-timeout/tsconfig.json\n",
            ),
            document(
                ".github/workflows/matrix.yml",
                "on: push\njobs:\n  invalid:\n    strategy:\n      matrix: {timeout: [0]}\n    timeout-minutes: '${{ matrix.timeout }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-matrix-timeout/tsconfig.json\n  valid:\n    strategy:\n      matrix: {timeout: [361]}\n    timeout-minutes: '${{ matrix.timeout }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid-matrix-timeout/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-timeout/tsconfig.json".to_string(),
        "invalid-matrix-timeout/tsconfig.json".to_string(),
        "valid-matrix-timeout/tsconfig.json".to_string(),
        "valid-timeout/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "valid-matrix-timeout/tsconfig.json".to_string(),
            "valid-timeout/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn skipped_need_results_are_available_to_continuation_conditions() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  setup:\n    if: false\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo skipped\n  skipped-result:\n    needs: setup\n    if: \"${{ always() && needs.setup.result == 'skipped' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p skipped-result/tsconfig.json\n  bracket-result:\n    needs: setup\n    if: \"${{ always() && needs['setup']['result'] == 'skipped' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p bracket-result/tsconfig.json\n  success-result:\n    needs: setup\n    if: \"${{ always() && needs.setup.result == 'success' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p success-result/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "bracket-result/tsconfig.json".to_string(),
        "skipped-result/tsconfig.json".to_string(),
        "success-result/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "bracket-result/tsconfig.json".to_string(),
            "skipped-result/tsconfig.json".to_string(),
        ])
    );
}
