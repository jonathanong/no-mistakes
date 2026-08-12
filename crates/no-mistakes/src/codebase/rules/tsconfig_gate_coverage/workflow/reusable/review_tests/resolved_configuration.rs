use super::*;

#[test]
fn exact_ref_names_are_type_checked_in_resolved_job_fields() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on:\n  push:\n    branches: [main]\njobs:\n  invalid-concurrency:\n    concurrency:\n      group: checks\n      cancel-in-progress: '${{ github.ref_name || false }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p concurrency/tsconfig.json\n  invalid-url:\n    environment:\n      name: production\n      url: '${{ case(true, fromJSON(github.ref_name), ''ok'') }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p environment/tsconfig.json\n  unknown-config:\n    if: vars.ENABLED\n    concurrency:\n      group: checks\n      cancel-in-progress: '${{ github.ref_name }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  dependent:\n    needs: unknown-config\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n",
        ), document(
            ".github/workflows/supported-checks.yml",
            "on:\n  push:\n    branches: [main]\njobs:\n  invalid-concurrency:\n    concurrency:\n      group: checks\n      cancel-in-progress: '${{ github.ref_name || false }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p concurrency/tsconfig.json\n  invalid-url:\n    environment:\n      name: production\n      url: '${{ fromJSON(github.ref_name) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p environment/tsconfig.json\n  unknown-config:\n    if: vars.ENABLED\n    concurrency:\n      group: checks\n      cancel-in-progress: '${{ github.ref_name }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  dependent:\n    needs: unknown-config\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "concurrency/tsconfig.json".to_string(),
        "dependent/tsconfig.json".to_string(),
        "environment/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::new(),
    );
}

#[test]
fn invalid_resolved_environment_urls_fail_dependent_jobs() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {url: '{}'}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {url: '\"https://example.test\"'}\n",
            ),
            document(
                ".github/workflows/invalid.yml",
                "on:\n  workflow_call:\n    inputs:\n      url: {type: string, required: true}\njobs:\n  deploy:\n    if: vars.ENABLED\n    environment:\n      name: production\n      url: '${{ fromJSON(inputs.url) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo deploy\n  dependent:\n    needs: deploy\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on:\n  workflow_call:\n    inputs:\n      url: {type: string, required: true}\njobs:\n  deploy:\n    environment:\n      name: production\n      url: '${{ fromJSON(inputs.url) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo deploy\n  dependent:\n    needs: deploy\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid/tsconfig.json".to_string(),
        "valid/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}

#[test]
fn non_stringable_environment_values_from_known_inputs_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n    with: {value: '{}'}\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      value: {type: string, required: true}\nenv:\n  DEPLOY_ENV: '${{ fromJSON(inputs.value) }}'\njobs:\n  typecheck:\n    environment: '${{ env.DEPLOY_ENV }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-environment/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["invalid-environment/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn invalid_resolved_step_timeouts_stop_later_steps() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {timeout: 0}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {timeout: 5}\n",
            ),
            document(
                ".github/workflows/invalid.yml",
                "on:\n  workflow_call:\n    inputs:\n      timeout: {type: number, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ inputs.timeout }}'\n        run: echo setup\n      - run: tsc --noEmit -p invalid-timeout/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on:\n  workflow_call:\n    inputs:\n      timeout: {type: number, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ inputs.timeout }}'\n        run: echo setup\n      - run: tsc --noEmit -p valid-timeout/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-timeout/tsconfig.json".to_string(),
        "valid-timeout/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid-timeout/tsconfig.json".to_string()])
    );
}

#[test]
fn unresolved_step_timeouts_propagate_indeterminate_outcomes() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - timeout-minutes: '${{ steps.prepare.outputs.timeout }}'\n        run: echo setup\n      - run: tsc --noEmit -p later/tsconfig.json\n  failure-handler:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "failure/tsconfig.json".to_string(),
        "later/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn job_timeout_validity_propagates_to_dependent_jobs() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/callee.yml\n    with: {timeout: 0}\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      timeout: {type: number, required: true}\njobs:\n  setup:\n    timeout-minutes: '${{ inputs.timeout }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  dependent:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid/tsconfig.json\n",
            ),
            document(
                ".github/workflows/unknown.yml",
                "on: push\njobs:\n  setup:\n    timeout-minutes: '${{ github.run_number }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  failure-handler:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "failure/tsconfig.json".to_string(),
        "invalid/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}
