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
fn missing_static_working_directories_fail_jobs_before_later_or_dependent_typechecks() {
    let workflow = document(
        ".github/workflows/missing-directory.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - working-directory: missing\n        run: tsc --noEmit -p ../setup/tsconfig.json\n      - run: tsc --noEmit -p later/tsconfig.json\n  dependent:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n",
    );

    assert!(scanned_projects(vec![workflow], &["setup", "later", "dependent"]).is_empty());
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
fn branch_exclusions_reach_only_nonexcluded_refs() {
    let workflow = document(
        ".github/workflows/branch-ignore.yml",
        "on:\n  push:\n    branches-ignore: [main]\njobs:\n  excluded:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p excluded-main/tsconfig.json\n  remaining:\n    if: github.ref == 'refs/heads/dev'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p remaining-branch/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["excluded-main", "remaining-branch"],),
        BTreeSet::from(["remaining-branch/tsconfig.json".to_string()])
    );
}

#[test]
fn exact_ref_filters_make_impossible_ref_conditions_unreachable() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/push.yml",
                "on:\n  push:\n    branches: [main]\n    tags: [v1]\njobs:\n  main:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-main/tsconfig.json\n  tag:\n    if: github.ref == 'refs/tags/v1'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-tag/tsconfig.json\n  dev:\n    if: github.ref == 'refs/heads/dev'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-dev/tsconfig.json\n",
            ),
            document(
                ".github/workflows/tag-only.yml",
                "on:\n  push:\n    tags: [v1]\njobs:\n  tag:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p tag-only/tsconfig.json\n",
            ),
            document(
                ".github/workflows/pull-request.yml",
                "on:\n  pull_request:\n    types: [synchronize]\n    branches: [main]\njobs:\n  base-ref:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-base-ref/tsconfig.json\n",
            ),
            document(
                ".github/workflows/pull-request-target.yml",
                "on:\n  pull_request_target:\n    types: [synchronize]\n    branches: [main]\njobs:\n  base-ref:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p pull-request-target-base-ref/tsconfig.json\n",
            ),
            document(
                ".github/workflows/ref-caller.yml",
                "on:\n  push:\n    branches: [main]\njobs:\n  call:\n    uses: ./.github/workflows/ref-callee.yml\n",
            ),
            document(
                ".github/workflows/ref-callee.yml",
                "on: workflow_call\njobs:\n  main:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p forwarded-main/tsconfig.json\n  dev:\n    if: github.ref == 'refs/heads/dev'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p forwarded-dev/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "push-main/tsconfig.json",
        "push-tag/tsconfig.json",
        "push-dev/tsconfig.json",
        "tag-only/tsconfig.json",
        "pull-request-base-ref/tsconfig.json",
        "pull-request-target-base-ref/tsconfig.json",
        "forwarded-main/tsconfig.json",
        "forwarded-dev/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "push-main/tsconfig.json".to_string(),
            "pull-request-target-base-ref/tsconfig.json".to_string(),
            "forwarded-main/tsconfig.json".to_string(),
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
