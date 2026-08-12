use super::test_support::collect_ci_projects_with_stats;
use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn workflow_document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

mod schema;

#[test]
fn invalid_non_reusable_trigger_configuration_earns_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/checks.yml",
                "on:\n  push:\n    paths: ['src/**']\n    paths-ignore: ['src/generated/**']\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-trigger/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/negative-only.yml",
                "on:\n  push:\n    paths: ['!src/generated/**']\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project negative-only/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/valid-trigger.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-trigger/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-trigger/tsconfig.json".to_string(),
        "negative-only/tsconfig.json".to_string(),
        "valid-trigger/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid-trigger/tsconfig.json".to_string()])
    );
}

#[test]
fn invalid_activity_type_and_cron_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/issues.yml",
                "on:\n  issues:\n    types: [not_an_issue_event]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-type/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/schedule.yml",
                "on:\n  schedule:\n    - cron: nope\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-cron/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/month-range.yml",
                "on:\n  schedule:\n    - cron: '0 0 * FEB-1 *'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-month-range/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-type/tsconfig.json".to_string(),
        "invalid-cron/tsconfig.json".to_string(),
        "invalid-month-range/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn unconfigured_or_expression_trigger_earns_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/schedule.yml",
                "on: schedule\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project scalar-schedule/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/push.yml",
                "on:\n  push:\n    paths: ['${{ github.event.repository.name }}/**']\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project expression-path/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "scalar-schedule/tsconfig.json".to_string(),
        "expression-path/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn zero_instance_matrix_needs_skip_dependents_unless_they_continue_explicitly() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  setup:\n    strategy:\n      matrix:\n        target: [ubuntu-latest]\n        exclude:\n          - target: ubuntu-latest\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo skipped\n  blocked:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project blocked/tsconfig.json\n  implicit-success:\n    needs: setup\n    if: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project implicit/tsconfig.json\n  continues:\n    needs: setup\n    if: '${{ always() }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project continues/tsconfig.json\n  compound-continues:\n    needs: setup\n    if: '${{ always() && true }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project compound/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "blocked/tsconfig.json".to_string(),
        "implicit/tsconfig.json".to_string(),
        "continues/tsconfig.json".to_string(),
        "compound/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "compound/tsconfig.json".to_string(),
            "continues/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn tenth_depth_remote_reusable_call_invalidates_sibling_projects() {
    let mut documents = vec![workflow_document(
        ".github/workflows/level-0.yml",
        "on: push\njobs:\n  call:\n    uses: ./.github/workflows/level-1.yml\n",
    )];
    for level in 1..10 {
        let yaml = if level == 9 {
            "on: workflow_call\njobs:\n  remote:\n    uses: octocat/repo/.github/workflows/checks.yml@v1\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sibling/tsconfig.json\n".to_string()
        } else {
            format!(
                "on: workflow_call\njobs:\n  call:\n    uses: ./.github/workflows/level-{}.yml\n",
                level + 1
            )
        };
        documents.push(workflow_document(
            &format!(".github/workflows/level-{level}.yml"),
            &yaml,
        ));
    }
    let workflows = ParsedWorkflowSet { documents };
    let tracked = BTreeSet::from(["sibling/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn skipped_reusable_calls_still_reject_cycles_before_sibling_projects() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/checks.yml",
                "on: push\njobs:\n  skipped-call:\n    if: false\n    uses: ./.github/workflows/first.yml\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sibling/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/first.yml",
                "on: workflow_call\njobs:\n  call:\n    uses: ./.github/workflows/second.yml\n",
            ),
            workflow_document(
                ".github/workflows/second.yml",
                "on: workflow_call\njobs:\n  call:\n    uses: ./.github/workflows/first.yml\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["sibling/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn skipped_acyclic_reusable_calls_validate_without_crediting_callee_projects() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/checks.yml",
                "on: push\njobs:\n  skipped-call:\n    if: false\n    uses: ./.github/workflows/callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/callee.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project callee/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["callee/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn zero_instance_reusable_calls_validate_once_without_crediting_callee_projects() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/checks.yml",
                "on: push\njobs:\n  skipped-call:\n    strategy:\n      matrix:\n        target: [linux]\n        exclude:\n          - target: linux\n    uses: ./.github/workflows/callee.yml\n    with:\n      enabled: true\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sibling/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project callee/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "callee/tsconfig.json".to_string(),
        "sibling/tsconfig.json".to_string(),
    ]);

    let (projects, computations) =
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked));
    assert_eq!(
        projects,
        BTreeSet::from(["sibling/tsconfig.json".to_string()])
    );
    assert_eq!(computations, 2);
}

#[test]
fn zero_instance_reusable_calls_reject_invalid_callee_boundary() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/checks.yml",
                "on: push\njobs:\n  skipped-call:\n    strategy:\n      matrix:\n        target: [linux]\n        exclude:\n          - target: linux\n    uses: ./.github/workflows/callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sibling/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project callee/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["sibling/tsconfig.json".to_string()]);

    let (projects, computations) =
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked));
    assert!(projects.is_empty());
    assert_eq!(computations, 1);
}

#[test]
fn uniform_matrix_values_control_steps_within_the_same_job() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  disabled:\n    strategy:\n      matrix:\n        enabled: [false]\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ matrix.enabled }}'\n        run: tsc --noEmit --project disabled-condition/tsconfig.json\n      - continue-on-error: '${{ matrix.enabled }}'\n        run: tsc --noEmit --project disabled-continue/tsconfig.json\n  enabled:\n    strategy:\n      matrix:\n        enabled: [true, true]\n        platform: [linux, macos]\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ matrix.enabled }}'\n        run: tsc --noEmit --project enabled-condition/tsconfig.json\n      - continue-on-error: '${{ matrix.enabled }}'\n        run: tsc --noEmit --project enabled-continue/tsconfig.json\n  mixed:\n    strategy:\n      matrix:\n        enabled: [false, true]\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ matrix.enabled }}'\n        run: tsc --noEmit --project mixed-condition/tsconfig.json\n      - continue-on-error: '${{ matrix.enabled }}'\n        run: tsc --noEmit --project mixed-continue/tsconfig.json\n  job-continue-disabled:\n    strategy:\n      matrix:\n        enabled: [false]\n    continue-on-error: '${{ matrix.enabled }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project job-continue-disabled/tsconfig.json\n  job-continue-enabled:\n    strategy:\n      matrix:\n        enabled: [true]\n    continue-on-error: '${{ matrix.enabled }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project job-continue-enabled/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "disabled-condition/tsconfig.json".to_string(),
        "disabled-continue/tsconfig.json".to_string(),
        "enabled-condition/tsconfig.json".to_string(),
        "enabled-continue/tsconfig.json".to_string(),
        "mixed-condition/tsconfig.json".to_string(),
        "mixed-continue/tsconfig.json".to_string(),
        "job-continue-disabled/tsconfig.json".to_string(),
        "job-continue-enabled/tsconfig.json".to_string(),
    ]);
    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "disabled-continue/tsconfig.json".to_string(),
            "enabled-condition/tsconfig.json".to_string(),
            "job-continue-disabled/tsconfig.json".to_string(),
            "mixed-condition/tsconfig.json".to_string(),
            "mixed-continue/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn matrix_values_stay_correlated_across_job_and_step_conditions() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  experimental:\n    strategy:\n      matrix:\n        experimental: [true, false]\n    continue-on-error: '${{ matrix.experimental }}'\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ matrix.experimental }}'\n        run: tsc --noEmit --project experimental/tsconfig.json\n  stable:\n    strategy:\n      matrix:\n        experimental: [true, false]\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ !matrix.experimental }}'\n        run: tsc --noEmit --project stable/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "experimental/tsconfig.json".to_string(),
        "stable/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["stable/tsconfig.json".to_string()])
    );
}

#[test]
fn static_matrix_timeouts_must_be_valid_before_steps_earn_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  valid:\n    strategy:\n      matrix: {timeout: [1, 360]}\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ matrix.timeout }}'\n        run: tsc --noEmit --project valid/tsconfig.json\n  invalid:\n    strategy:\n      matrix: {timeout: [0, 361]}\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ matrix.timeout }}'\n        run: tsc --noEmit --project invalid/tsconfig.json\n  dynamic:\n    strategy:\n      matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ matrix.timeout }}'\n        run: tsc --noEmit --project dynamic/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "valid/tsconfig.json".to_string(),
        "invalid/tsconfig.json".to_string(),
        "dynamic/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}

#[test]
fn static_matrix_images_must_be_valid_before_steps_earn_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  invalid-container:\n    strategy:\n      matrix: {tag: [':']}\n    runs-on: ubuntu-latest\n    container: 'node:${{ matrix.tag }}'\n    steps:\n      - run: tsc --noEmit --project invalid-container/tsconfig.json\n  empty-container:\n    strategy:\n      matrix: {image: ['']}\n    runs-on: ubuntu-latest\n    container: '${{ matrix.image }}'\n    steps:\n      - run: tsc --noEmit --project empty-container/tsconfig.json\n  valid-container:\n    strategy:\n      matrix: {tag: [22]}\n    runs-on: ubuntu-latest\n    container: 'node:${{ matrix.tag }}'\n    steps:\n      - run: tsc --noEmit --project valid-container/tsconfig.json\n  invalid-service:\n    strategy:\n      matrix: {tag: [':']}\n    runs-on: ubuntu-latest\n    services:\n      postgres: {image: 'postgres:${{ matrix.tag }}'}\n    steps:\n      - run: tsc --noEmit --project invalid-service/tsconfig.json\n  empty-service:\n    strategy:\n      matrix: {image: ['']}\n    runs-on: ubuntu-latest\n    services:\n      postgres: {image: '${{ matrix.image }}'}\n    steps:\n      - run: tsc --noEmit --project empty-service/tsconfig.json\n  dynamic:\n    strategy:\n      matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'\n    runs-on: ubuntu-latest\n    container: 'node:${{ matrix.tag }}'\n    steps:\n      - run: tsc --noEmit --project dynamic-image/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "invalid-container/tsconfig.json".to_string(),
        "empty-container/tsconfig.json".to_string(),
        "valid-container/tsconfig.json".to_string(),
        "invalid-service/tsconfig.json".to_string(),
        "empty-service/tsconfig.json".to_string(),
        "dynamic-image/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "valid-container/tsconfig.json".to_string(),
            "empty-service/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn bracketed_matrix_conditions_preserve_static_gate_boundaries() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  skipped:\n    strategy:\n      matrix:\n        enabled: [false]\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ matrix [ ''enabled'' ] }}'\n        run: tsc --noEmit --project skipped/tsconfig.json\n  enforced:\n    strategy:\n      matrix:\n        enabled: [true]\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ matrix [ ''ENABLED'' ] }}'\n        run: tsc --noEmit --project enforced/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "skipped/tsconfig.json".to_string(),
        "enforced/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["enforced/tsconfig.json".to_string()])
    );
}

#[test]
fn literal_expression_matrix_axes_control_step_gate_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  disabled:\n    strategy:\n      matrix:\n        enabled: ['${{ false }}']\n    runs-on: ubuntu-latest\n    steps:\n      - if: matrix.enabled\n        run: tsc --noEmit --project literal-disabled/tsconfig.json\n  enabled:\n    strategy:\n      matrix:\n        enabled: ['${{ true }}']\n    runs-on: ubuntu-latest\n    steps:\n      - if: matrix.enabled\n        run: tsc --noEmit --project literal-enabled/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "literal-disabled/tsconfig.json".to_string(),
        "literal-enabled/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["literal-enabled/tsconfig.json".to_string()])
    );
}

#[test]
fn static_step_failures_bound_pipeline_credits_and_later_step_outcome_conditions() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      # A pipefail failure terminates this step, but the preceding check did run.\n      - shell: bash -eo pipefail {0}\n        run: |\n          tsc --noEmit --project before-pipeline/tsconfig.json\n          false | true\n          tsc --noEmit --project after-pipeline/tsconfig.json\n  outcomes:\n    runs-on: ubuntu-latest\n    steps:\n      - id: tolerated\n        continue-on-error: true\n        run: 'false'\n      - if: \"${{ steps.tolerated.outcome == 'success' }}\"\n        run: tsc --noEmit --project failed-outcome/tsconfig.json\n      - if: \"${{ steps.tolerated.outcome == 'failure' }}\"\n        continue-on-error: \"${{ steps.tolerated.outcome == 'failure' }}\"\n        run: tsc --noEmit --project outcome-tolerated/tsconfig.json\n  known-success:\n    runs-on: ubuntu-latest\n    steps:\n      - id: setup\n        run: 'true'\n      - if: \"${{ steps.setup.outcome == 'failure' }}\"\n        run: tsc --noEmit --project success-outcome/tsconfig.json\n  skipped:\n    runs-on: ubuntu-latest\n    steps:\n      - id: disabled\n        if: false\n        run: tsc --noEmit --project disabled/tsconfig.json\n      - if: \"${{ steps.disabled.outcome != 'skipped' }}\"\n        run: tsc --noEmit --project skipped-outcome/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "before-pipeline/tsconfig.json".to_string(),
        "after-pipeline/tsconfig.json".to_string(),
        "failed-outcome/tsconfig.json".to_string(),
        "outcome-tolerated/tsconfig.json".to_string(),
        "success-outcome/tsconfig.json".to_string(),
        "disabled/tsconfig.json".to_string(),
        "skipped-outcome/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["before-pipeline/tsconfig.json".to_string()])
    );
}

#[test]
fn step_conclusions_distinguish_tolerated_failures_and_successful_actions() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  tolerated:\n    runs-on: ubuntu-latest\n    steps:\n      - id: test\n        continue-on-error: true\n        run: 'false'\n      - if: \"${{ steps.test.outcome == 'failure' && steps.test.conclusion == 'success' }}\"\n        run: tsc --noEmit --project tolerated/tsconfig.json\n  action:\n    runs-on: ubuntu-latest\n    steps:\n      - id: checkout\n        uses: actions/checkout@v4\n      - if: \"${{ steps.checkout.outcome == 'success' && steps.checkout.conclusion == 'success' }}\"\n        run: tsc --noEmit --project action/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "tolerated/tsconfig.json".to_string(),
        "action/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        tracked
    );
}

#[test]
fn prior_step_outcomes_resolve_when_later_step_environment_is_built() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - id: setup\n        run: 'true; exit'\n      - env:\n          SETUP_OUTCOME: '${{ steps.setup.outcome }}'\n        if: \"${{ env.SETUP_OUTCOME == 'failure' }}\"\n        run: tsc --noEmit --project should-not-run/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from(["should-not-run/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn pipefail_after_an_unknown_command_stops_later_steps_but_keeps_the_prior_check() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash -eo pipefail {0}\n        run: tsc --noEmit --project before/tsconfig.json && false | true\n      - run: tsc --noEmit --project after/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "before/tsconfig.json".to_string(),
        "after/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["before/tsconfig.json".to_string()])
    );
}
