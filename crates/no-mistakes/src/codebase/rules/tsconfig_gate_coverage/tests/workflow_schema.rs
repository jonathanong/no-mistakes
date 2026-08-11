use super::*;

fn workflow(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

#[test]
fn invalid_contract_jobs_dependencies_and_empty_matrices_earn_no_credit() {
    let oversized_values = (0..257)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let documents = vec![
        workflow(
            ".github/workflows/declaration-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/declaration-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project declaration/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/declaration-callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        required: 'true'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project declaration/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/jobs-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/jobs-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project jobs/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/jobs-callee.yml",
            "on: workflow_call\njobs: []\n",
        ),
        workflow(
            ".github/workflows/cycle.yml",
            "on: push\njobs:\n  first:\n    needs: second\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project cycle/tsconfig.json\n  second:\n    needs: first\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo blocked\n",
        ),
        workflow(
            ".github/workflows/excluded.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux]\n        exclude:\n          - target: linux\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project excluded/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/partial.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux, macos]\n        exclude:\n          - target: linux\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project partial/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/oversized-matrix.yml",
            &format!(
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        value: [{oversized_values}]\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project oversized/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project oversized-sibling/tsconfig.json\n"
            ),
        ),
        workflow(
            ".github/workflows/ambiguous-step.yml",
            "on: push\njobs:\n  ambiguous:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ambiguous/tsconfig.json\n        uses: owner/action@v1\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ambiguous-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-remote-bindings.yml",
            "on: push\njobs:\n  remote:\n    uses: owner/repository/.github/workflows/typecheck.yml@main\n    with: true\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project remote-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-strategy.yml",
            "on: push\njobs:\n  malformed:\n    strategy: []\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-strategy/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-strategy-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/misspelled-trigger-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/misspelled-trigger-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/misspelled-trigger-callee.yml",
            "on:\n  workflow_call:\n  workflow_dispath:\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/sequence-misspelled-trigger-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/sequence-misspelled-trigger-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sequence-misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/sequence-misspelled-trigger-callee.yml",
            "on: [workflow_call, workflow_dispath]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sequence-misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-action.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout\n      - run: tsc --noEmit --project malformed-action/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-action-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-action-ref.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@bad..ref\n      - run: tsc --noEmit --project invalid-action-ref/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-action-ref-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-action-lock-ref.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@refs/heads/release.lock\n      - run: tsc --noEmit --project invalid-action-lock-ref/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-action-lock-ref-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-container-ports.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n    container: {image: node:22, ports: [true]}\n    steps:\n      - run: tsc --noEmit --project invalid-container-ports/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-service-ports.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n    services: {postgres: {image: postgres:16, ports: [false]}}\n    steps:\n      - run: tsc --noEmit --project invalid-service-ports/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-ports.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: node:22, ports: [8080, '8081:80']}\n    services: {postgres: {image: postgres:16, ports: ['127.0.0.1:5432:5432/tcp']}}\n    steps:\n      - run: tsc --noEmit --project valid-ports/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/missing-steps.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project missing-steps-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/missing-runs-on-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/missing-runs-on-callee.yml\n",
        ),
        workflow(
            ".github/workflows/missing-runs-on-callee.yml",
            "on: workflow_call\njobs:\n  malformed:\n    steps:\n      - run: echo invalid\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project missing-runs-on-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/webhook-only-trigger-caller.yml",
            "on:\n  push:\n  repository:\njobs:\n  typecheck:\n    uses: ./.github/workflows/webhook-only-trigger-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project webhook-only-trigger-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/webhook-only-trigger-callee.yml",
            "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project webhook-only-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/uppercase-inherit.yml",
            "on: push\njobs:\n  remote:\n    uses: owner/repository/.github/workflows/typecheck.yml@main\n    secrets: INHERIT\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project uppercase-inherit-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/empty-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: '${{ }}'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/concatenated-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: '${{ true }}${{ false }}'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project concatenated-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/incomplete-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: '${{ true && }}'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project incomplete-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/literal-type-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: \"${{ 'false' }}\"\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project literal-type-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/literal-postfix-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: '${{ 1.foo }}'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project literal-postfix-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-expression-callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        required: true\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-expression-callee/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-trigger-config.yml",
            "on:\n  push: []\njobs:\n  call:\n    uses: ./.github/workflows/malformed-trigger-config-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-trigger-config-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-trigger-config-callee.yml",
            "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-trigger-config/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/empty-steps-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/empty-steps-callee.yml\n",
        ),
        workflow(
            ".github/workflows/empty-steps-callee.yml",
            "on: workflow_call\njobs:\n  empty:\n    runs-on: ubuntu-latest\n    steps: []\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-steps-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unknown-job-key.yml",
            "on: push\njobs:\n  invalid:\n    bogus: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo invalid\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unknown-job-key-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unknown-step-key.yml",
            "on: push\njobs:\n  invalid:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo invalid\n        bogus: true\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unknown-step-key-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/duplicate-step-id-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/duplicate-step-id-callee.yml\n",
        ),
        workflow(
            ".github/workflows/duplicate-step-id-callee.yml",
            "on: workflow_call\njobs:\n  invalid:\n    runs-on: ubuntu-latest\n    steps:\n      - id: Build\n        run: echo invalid\n      - id: build\n        run: echo invalid\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project duplicate-step-id-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/scalar-matrix-axis.yml",
            "on: push\njobs:\n  invalid:\n    strategy:\n      matrix:\n        os: ubuntu-latest\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo invalid\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project scalar-matrix-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-step-condition.yml",
            "on: push\njobs:\n  invalid:\n    runs-on: ubuntu-latest\n    steps:\n      - if: '${{ }}'\n        run: echo invalid\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-step-condition-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-remote-repository.yml",
            "on: push\njobs:\n  invalid:\n    uses: octo/../.github/workflows/checks.yml@main\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-remote-repository-sibling/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "declaration",
        "jobs",
        "cycle",
        "excluded",
        "partial",
        "oversized",
        "oversized-sibling",
        "ambiguous",
        "ambiguous-sibling",
        "remote-sibling",
        "malformed-strategy",
        "malformed-strategy-sibling",
        "misspelled-trigger",
        "sequence-misspelled-trigger",
        "malformed-action",
        "malformed-action-sibling",
        "invalid-action-ref",
        "invalid-action-ref-sibling",
        "invalid-action-lock-ref",
        "invalid-action-lock-ref-sibling",
        "invalid-container-ports",
        "invalid-service-ports",
        "valid-ports",
        "missing-steps-sibling",
        "missing-runs-on-sibling",
        "webhook-only-trigger",
        "webhook-only-trigger-sibling",
        "uppercase-inherit-sibling",
        "empty-expression-sibling",
        "concatenated-expression-sibling",
        "incomplete-expression-sibling",
        "literal-type-expression-sibling",
        "literal-postfix-expression-sibling",
        "malformed-expression-callee",
        "malformed-trigger-config",
        "malformed-trigger-config-sibling",
        "empty-steps-sibling",
        "unknown-job-key-sibling",
        "unknown-step-key-sibling",
        "duplicate-step-id-sibling",
        "scalar-matrix-sibling",
        "malformed-step-condition-sibling",
        "invalid-remote-repository-sibling",
    ]
    .into_iter()
    .map(|project| format!("{project}/tsconfig.json"))
    .collect();

    assert_eq!(
        ci_typechecked_projects(
            &ParsedWorkflowSet { documents },
            &tracked,
            &project_inputs(&tracked)
        ),
        BTreeSet::from([
            "partial/tsconfig.json".to_string(),
            "valid-ports/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn literal_expression_exclusions_create_a_zero_instance_workflow_boundary() {
    let tracked = BTreeSet::from(["excluded-expression/tsconfig.json".to_string()]);
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow(
            ".github/workflows/excluded-expression.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        enabled: [true]\n        exclude:\n          - enabled: '${{ true }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project excluded-expression/tsconfig.json\n",
        )],
    };
    assert!(ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)).is_empty());
}

#[test]
fn scalar_literal_matrix_mapping_expressions_earn_no_coverage() {
    let tracked = BTreeSet::from([
        "scalar-include/tsconfig.json".to_string(),
        "scalar-exclude/tsconfig.json".to_string(),
        "scalar-function/tsconfig.json".to_string(),
        "valid-include/tsconfig.json".to_string(),
    ]);
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow(
                ".github/workflows/scalar-include.yml",
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux]\n        include: '${{ true }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project scalar-include/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/scalar-exclude.yml",
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux]\n        exclude: '${{ null }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project scalar-exclude/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/scalar-function.yml",
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux]\n        include: \"${{ contains('matrix', 'm') }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project scalar-function/tsconfig.json\n",
            ),
            workflow(
                ".github/workflows/valid-include.yml",
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux]\n        include:\n          - target: macos\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-include/tsconfig.json\n",
            ),
        ],
    };
    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from(["valid-include/tsconfig.json".to_string()])
    );
}

#[test]
fn workflow_level_expression_schema_errors_do_not_credit_typechecks() {
    let documents = vec![
        workflow(
            ".github/workflows/unavailable-workflow-env.yml",
            "on: push\nenv:\n  TYPECHECK_REF: '${{ jobs.typecheck.outputs.ref }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unavailable-workflow-env/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/dynamic-defaults.yml",
            "on: push\ndefaults:\n  run:\n    working-directory: 'packages/${{ inputs.package }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic-defaults/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-workflow-env.yml",
            "on: push\nenv:\n  TYPECHECK_REF: '${{ github.ref }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-workflow-env/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unavailable-concurrency.yml",
            "on: push\nconcurrency:\n  group: checks-${{ needs.setup.outputs.key }}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unavailable-concurrency/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid.yml",
            "on: push\nconcurrency:\n  group: checks-${{ github.ref }}\n  cancel-in-progress: '${{ vars.CANCEL }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "dynamic-defaults/tsconfig.json",
        "unavailable-concurrency/tsconfig.json",
        "unavailable-workflow-env/tsconfig.json",
        "valid/tsconfig.json",
        "valid-workflow-env/tsconfig.json",
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
        BTreeSet::from([
            "valid/tsconfig.json".to_string(),
            "valid-workflow-env/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn job_level_expression_schema_errors_do_not_credit_typechecks() {
    let documents = vec![
        workflow(
            ".github/workflows/invalid-action-input-context.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: '${{ jobs.typecheck.outputs.ref }}'\n      - run: tsc --noEmit --project invalid-action-input-context/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-action-input-function.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\n      - run: tsc --noEmit --project valid-action-input-function/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-action-input-context.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: \"${{ format('{0}', github.ref) }}\"\n      - run: tsc --noEmit --project valid-action-input-context/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/dynamic-job-defaults.yml",
            "on: push\njobs:\n  typecheck:\n    defaults:\n      run:\n        shell: '${{ github.ref }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic-job-defaults/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unavailable-job-concurrency.yml",
            "on: push\njobs:\n  typecheck:\n    concurrency: checks-${{ secrets.TOKEN }}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unavailable-job-concurrency/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-job-timeout.yml",
            "on: push\njobs:\n  typecheck:\n    timeout-minutes: '${{ failure() }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-job-timeout/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-step-timeout.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ failure() }}'\n        run: tsc --noEmit --project invalid-step-timeout/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-job-contexts.yml",
            "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    outputs:\n      key: value\n    steps:\n      - run: echo setup\n  typecheck:\n    needs: setup\n    concurrency:\n      group: checks-${{ needs.setup.outputs.key }}\n      cancel-in-progress: '${{ github.ref_protected }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-job-contexts/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "dynamic-job-defaults/tsconfig.json",
        "invalid-job-timeout/tsconfig.json",
        "invalid-action-input-context/tsconfig.json",
        "invalid-step-timeout/tsconfig.json",
        "unavailable-job-concurrency/tsconfig.json",
        "valid-job-contexts/tsconfig.json",
        "valid-action-input-context/tsconfig.json",
        "valid-action-input-function/tsconfig.json",
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
        BTreeSet::from([
            "valid-action-input-context/tsconfig.json".to_string(),
            "valid-action-input-function/tsconfig.json".to_string(),
            "valid-job-contexts/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn scalar_root_matrix_expressions_do_not_credit_typechecks() {
    let scalar_expressions = [
        "true",
        "42",
        "'matrix'",
        "null",
        "contains('matrix', 'm')",
        "startsWith('matrix', 'm')",
        "success()",
        "toJSON(github)",
        "true || false",
    ];
    let mut documents = scalar_expressions
        .iter()
        .enumerate()
        .map(|(index, expression)| {
            let project = format!("scalar-matrix-{index}");
            workflow(
                &format!(".github/workflows/{project}.yml"),
                &format!(
                    "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix: \"${{{{ {expression} }}}}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project {project}/tsconfig.json\n"
                ),
            )
        })
        .collect::<Vec<_>>();
    for (name, expression) in [
        ("dynamic-from-json", "fromJSON(needs.setup.outputs.matrix)"),
        ("dynamic-property", "needs.setup.outputs.matrix"),
        (
            "dynamic-case",
            "case(inputs.enabled, fromJSON(inputs.matrix), needs.setup.outputs.matrix)",
        ),
    ] {
        documents.push(workflow(
            &format!(".github/workflows/{name}.yml"),
            &format!(
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix: \"${{{{ {expression} }}}}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project {name}/tsconfig.json\n"
            ),
        ));
    }
    let tracked = scalar_expressions
        .iter()
        .enumerate()
        .map(|(index, _)| format!("scalar-matrix-{index}/tsconfig.json"))
        .chain(
            ["dynamic-from-json", "dynamic-property", "dynamic-case"]
                .into_iter()
                .map(|name| format!("{name}/tsconfig.json")),
        )
        .collect();

    assert_eq!(
        ci_typechecked_projects(
            &ParsedWorkflowSet { documents },
            &tracked,
            &project_inputs(&tracked),
        ),
        BTreeSet::from([
            "dynamic-case/tsconfig.json".to_string(),
            "dynamic-from-json/tsconfig.json".to_string(),
            "dynamic-property/tsconfig.json".to_string(),
        ])
    );
}
