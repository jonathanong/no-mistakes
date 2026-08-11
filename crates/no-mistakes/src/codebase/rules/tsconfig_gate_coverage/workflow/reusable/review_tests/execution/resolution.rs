use super::*;

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
            "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {environment: ''}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {environment: staging}\n  sequence:\n    uses: ./.github/workflows/non-stringable.yml\n    with: {environment: '[]'}\n  mapping:\n    uses: ./.github/workflows/non-stringable.yml\n    with: {environment: '{\"name\":\"production\"}'}\n  invalid-json:\n    uses: ./.github/workflows/non-stringable.yml\n    with: {environment: not-json}\n",
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
        document(
            ".github/workflows/non-stringable.yml",
            "on:\n  workflow_call:\n    inputs:\n      environment: {type: string, required: true}\njobs:\n  sequence:\n    environment: '${{ fromJSON(inputs.environment) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p sequence-environment/tsconfig.json\n  mapping:\n    environment: '${{ fromJSON(inputs.environment) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p mapping-environment/tsconfig.json\n",
        ),
        document(
            ".github/workflows/missing-input.yml",
            "on: push\njobs:\n  missing:\n    environment: '${{ fromJSON(inputs.missing) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p missing-environment/tsconfig.json\n",
        ),
        document(
            ".github/workflows/dynamic-input.yml",
            "on: push\njobs:\n  dynamic:\n    environment: '${{ fromJSON(github.event.inputs.environment) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dynamic-environment/tsconfig.json\n",
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
                "sequence-environment",
                "mapping-environment",
                "missing-environment",
                "dynamic-environment",
            ],
        ),
        BTreeSet::from([
            "valid-environment/tsconfig.json".to_string(),
            "valid-matrix-environment/tsconfig.json".to_string(),
            "dynamic-environment/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn caller_resolved_root_matrices_enforce_the_job_limit_before_expansion() {
    let caller_resolved_projects = |count| {
        let matrix = serde_json::json!({"value": (0..count).collect::<Vec<_>>()});
        let caller = document(
            ".github/workflows/caller.yml",
            &format!(
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n    with: {{matrix: '{matrix}'}}\n"
            ),
        );
        let callee = document(
            ".github/workflows/callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      matrix: {type: string, required: true}\njobs:\n  typecheck:\n    strategy:\n      matrix: '${{ fromJSON(inputs.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p matrix-limit/tsconfig.json\n",
        );

        scanned_projects(vec![caller, callee], &["matrix-limit"])
    };

    assert_eq!(
        caller_resolved_projects(256),
        BTreeSet::from(["matrix-limit/tsconfig.json".to_string()])
    );
    assert!(caller_resolved_projects(257).is_empty());
}

#[test]
fn caller_resolved_zero_instance_matrices_skip_downstream_jobs() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n    with: {matrix: '{\"target\":[\"linux\"],\"exclude\":[{\"target\":\"linux\"}]}'}\n",
    );
    let callee = document(
        ".github/workflows/callee.yml",
        "on:\n  workflow_call:\n    inputs:\n      matrix: {type: string, required: true}\njobs:\n  setup:\n    strategy:\n      matrix: '${{ fromJSON(inputs.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo skipped\n  blocked:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked/tsconfig.json\n  continues:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p continues/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![caller, callee], &["blocked", "continues"]),
        BTreeSet::from(["continues/tsconfig.json".to_string()])
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
