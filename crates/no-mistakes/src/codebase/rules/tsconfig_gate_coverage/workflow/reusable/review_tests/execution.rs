use super::*;

fn scanned_projects(documents: Vec<ParsedWorkflowDocument>, projects: &[&str]) -> BTreeSet<String> {
    let tracked = projects
        .iter()
        .map(|project| format!("{project}/tsconfig.json"))
        .collect::<BTreeSet<_>>();
    collect_ci_projects_with_stats(
        &ParsedWorkflowSet { documents },
        &tracked,
        &project_inputs(&tracked),
    )
    .0
}

#[test]
fn static_step_failures_only_allow_explicit_continuations() {
    let workflow = document(
        ".github/workflows/steps.yml",
        "on: push\njobs:\n  blocked:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n      - if: true\n        run: tsc --noEmit -p blocked/tsconfig.json\n  always:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false'\n      - if: always()\n        run: tsc --noEmit -p always/tsconfig.json\n  failure:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false'\n      - if: failure()\n        run: tsc --noEmit -p failure/tsconfig.json\n  tolerated:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: true\n        run: 'false'\n      - run: tsc --noEmit -p tolerated/tsconfig.json\n  non-errexit-failure:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: exit 1\n      - run: tsc --noEmit -p non-errexit-failure/tsconfig.json\n  non-errexit-success:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: 'false; echo ok'\n      - run: tsc --noEmit -p non-errexit-success/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "blocked",
                "always",
                "failure",
                "tolerated",
                "non-errexit-failure",
                "non-errexit-success",
            ],
        ),
        BTreeSet::from([
            "always/tsconfig.json".to_string(),
            "failure/tsconfig.json".to_string(),
            "non-errexit-success/tsconfig.json".to_string(),
            "tolerated/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn bare_exit_preserves_the_preceding_command_status() {
    let workflow = document(
        ".github/workflows/bare-exit.yml",
        "on: push\njobs:\n  failed:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: 'false; exit'\n      - run: tsc --noEmit -p failed/tsconfig.json\n  succeeded:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: 'true; exit'\n      - run: tsc --noEmit -p succeeded/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["failed", "succeeded"]),
        BTreeSet::from(["succeeded/tsconfig.json".to_string()])
    );
}

#[test]
fn static_job_failures_propagate_through_needs() {
    let workflow = document(
        ".github/workflows/job-failures.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n  ordinary:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p ordinary/tsconfig.json\n  literal:\n    needs: setup\n    if: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p literal/tsconfig.json\n  transitive:\n    needs: ordinary\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p transitive/tsconfig.json\n  always:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always/tsconfig.json\n  failure-handler:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-handler/tsconfig.json\n      - run: exit 1\n  after-handler:\n    needs: failure-handler\n    if: always() && needs.failure-handler.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p after-handler/tsconfig.json\n  failure-result:\n    needs: setup\n    if: always() && needs.setup.result == 'failure'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-result/tsconfig.json\n  tolerated:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: true\n        run: exit 1\n  after-tolerated:\n    needs: tolerated\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p after-tolerated/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "ordinary",
                "literal",
                "transitive",
                "always",
                "failure-handler",
                "after-handler",
                "failure-result",
                "after-tolerated",
            ],
        ),
        BTreeSet::from([
            "after-tolerated/tsconfig.json".to_string(),
            "after-handler/tsconfig.json".to_string(),
            "always/tsconfig.json".to_string(),
            "failure-result/tsconfig.json".to_string(),
        ])
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

#[test]
fn static_step_working_directories_and_condition_budgets_bound_coverage() {
    let over_budget = std::iter::repeat_n("true", 257)
        .collect::<Vec<_>>()
        .join(" && ");
    let directory = document(
        ".github/workflows/directory.yml",
        "on: push\njobs:\n  directory:\n    runs-on: ubuntu-latest\n    steps:\n      - working-directory: \"${{ 'package' }}\"\n        run: tsc --noEmit -p tsconfig.json\n",
    );
    let over_budget = document(
        ".github/workflows/over-budget.yml",
        &format!(
            "on: push\njobs:\n  over-budget:\n    runs-on: ubuntu-latest\n    steps:\n      - if: ${{{{ {over_budget} }}}}\n        run: tsc --noEmit -p over-budget/tsconfig.json\n"
        ),
    );

    assert_eq!(
        scanned_projects(vec![directory, over_budget], &["package", "over-budget"]),
        BTreeSet::from(["package/tsconfig.json".to_string()])
    );
}

#[test]
fn resolved_environment_names_must_be_nonempty_per_activation() {
    let documents = vec![
        document(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {environment: ''}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {environment: staging}\n",
        ),
        document(
            ".github/workflows/invalid.yml",
            "on:\n  workflow_call:\n    inputs:\n      environment: {type: string, required: true}\njobs:\n  typecheck:\n    environment: '${{ inputs.environment }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-environment/tsconfig.json\n",
        ),
        document(
            ".github/workflows/valid.yml",
            "on:\n  workflow_call:\n    inputs:\n      environment: {type: string, required: true}\njobs:\n  typecheck:\n    environment: {name: '${{ inputs.environment }}'}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid-environment/tsconfig.json\n",
        ),
        document(
            ".github/workflows/matrix.yml",
            "on: push\njobs:\n  invalid:\n    strategy:\n      matrix: {environment: ['']}\n    environment: {name: '${{ matrix.environment }}'}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-matrix-environment/tsconfig.json\n  valid:\n    strategy:\n      matrix: {environment: [production]}\n    environment: '${{ matrix.environment }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid-matrix-environment/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned_projects(
            documents,
            &[
                "invalid-environment",
                "valid-environment",
                "invalid-matrix-environment",
                "valid-matrix-environment",
            ],
        ),
        BTreeSet::from([
            "valid-environment/tsconfig.json".to_string(),
            "valid-matrix-environment/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn masked_failures_and_skipped_needs_preserve_distinct_statuses() {
    let workflow = document(
        ".github/workflows/statuses.yml",
        "on: push\njobs:\n  masked:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false && echo masked; echo completed'\n      - run: tsc --noEmit -p masked/tsconfig.json\n  setup:\n    if: false\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo skipped\n  failure:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure-after-skip/tsconfig.json\n  always:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always-after-skip/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &["masked", "failure-after-skip", "always-after-skip"],
        ),
        BTreeSet::from([
            "always-after-skip/tsconfig.json".to_string(),
            "masked/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn job_default_directories_resolve_per_matrix_combination() {
    let workflow = document(
        ".github/workflows/job-directory.yml",
        "on: push\njobs:\n  directory:\n    strategy:\n      matrix: {package: [job-package]}\n    defaults:\n      run:\n        working-directory: '${{ matrix.package }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["job-package"]),
        BTreeSet::from(["job-package/tsconfig.json".to_string()])
    );
}

#[test]
fn failure_propagating_shell_constructs_block_later_steps() {
    let workflow = document(
        ".github/workflows/failure-propagation.yml",
        "on: push\njobs:\n  errexit:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'set -e; false; echo unreachable'\n      - run: tsc --noEmit -p errexit/tsconfig.json\n  pipefail:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'false | true; echo unreachable'\n      - run: tsc --noEmit -p pipefail/tsconfig.json\n  recovered:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false | true || echo recovered'\n      - run: tsc --noEmit -p recovered/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["errexit", "pipefail", "recovered"]),
        BTreeSet::from(["recovered/tsconfig.json".to_string()])
    );
}

#[test]
fn pipefail_preserves_final_pipeline_and_and_list_status_without_errexit() {
    let workflow = document(
        ".github/workflows/custom-pipefail.yml",
        "on: push\njobs:\n  pipeline:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -o pipefail {0}'\n        run: 'false | true'\n      - run: tsc --noEmit -p pipeline/tsconfig.json\n  and-list:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'false | true && echo masked'\n      - run: tsc --noEmit -p and-list/tsconfig.json\n  completed:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false | true && echo masked; echo completed'\n      - run: tsc --noEmit -p completed/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["pipeline", "and-list", "completed"]),
        BTreeSet::from(["completed/tsconfig.json".to_string()])
    );
}

#[test]
fn pipefail_tracks_reachable_pipelines_later_in_and_lists() {
    let workflow = document(
        ".github/workflows/later-pipeline.yml",
        "on: push\njobs:\n  final-pipeline:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -o pipefail {0}'\n        run: 'true | true && false | true'\n      - run: tsc --noEmit -p final-pipeline/tsconfig.json\n",
    );

    assert!(scanned_projects(vec![workflow], &["final-pipeline"]).is_empty());
}
