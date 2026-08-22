use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{MatrixState, direct_inputs, inputs_with_matrix_values},
    reusable::model::{GithubEventContext, GithubRef},
};
use crate::codebase::{
    ci_graph::parse::parse_workflow_value, rules::tsconfig_gate_coverage::ProjectSourceInputs,
};

fn scan(job: &str, local_actions: BTreeSet<String>) -> StepScan {
    scan_with_catalog(
        job,
        super::super::super::local_actions::LocalActionCatalog::non_docker(local_actions),
    )
}

fn scan_with_catalog(
    job: &str,
    local_actions: super::super::super::local_actions::LocalActionCatalog,
) -> StepScan {
    let job = serde_yaml::from_str(job).unwrap();
    let workflow: Value = serde_yaml::from_str("'on': push").unwrap();
    let model = parse_workflow_value(&workflow, ".github/workflows/test.yml");
    let triggers = CompiledTriggers::for_event(&model, "push").unwrap();
    let tracked = BTreeSet::new();
    let source_inputs = ProjectSourceInputs::new();
    let context = ScanContext {
        workflows: Default::default(),
        tracked: &tracked,
        visible_paths: BTreeSet::from([".".to_string()]),
        project_source_inputs: &source_inputs,
        local_actions: &local_actions,
    };

    let inputs = direct_inputs(
        None,
        &GithubEventContext::with_ref("push", GithubRef::Unknown),
    )
    .unwrap();
    let inputs = inputs_with_matrix_values(&inputs, &Default::default(), MatrixState::Dynamic);
    let environment = EnvironmentState::default()
        .with_runner_os(super::super::super::runtime::runner_os(&job, &inputs));
    scan_job_steps(&job, &triggers, &inputs, &environment, None, None, &context)
}

#[test]
fn direct_step_scanning_fail_closes_invalid_and_unresolved_runtime_states() {
    let none = scan("runs-on: ubuntu-latest", BTreeSet::new());
    assert!(!none.failed && !none.indeterminate && none.projects.is_empty());

    let invalid_environment = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - env: {VALUE: \"${{ fromJSON('[]') }}\"}\n    run: echo invalid",
        BTreeSet::new(),
    );
    assert!(invalid_environment.failed);
    let uncertain_environment = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - if: github.event.unknown\n    env: {VALUE: \"${{ fromJSON('[]') }}\"}\n    run: echo invalid",
        BTreeSet::new(),
    );
    assert!(uncertain_environment.indeterminate);

    let invalid_action_input = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - uses: actions/checkout@v4\n    with: {ref: \"${{ fromJSON('{}') }}\"}",
        BTreeSet::new(),
    );
    assert!(invalid_action_input.failed);
    let tolerated_invalid_action = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    uses: actions/checkout@v4\n    with: {ref: \"${{ fromJSON('{}') }}\"}",
        BTreeSet::new(),
    );
    assert!(!tolerated_invalid_action.failed && !tolerated_invalid_action.indeterminate);

    let unavailable = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - uses: ./local-action",
        BTreeSet::from(["local-action".to_string()]),
    );
    assert!(unavailable.failed);

    let uncertain_local = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - if: github.event.unknown\n    uses: ./local-action",
        BTreeSet::from(["local-action".to_string()]),
    );
    assert!(uncertain_local.indeterminate);
}

#[test]
fn direct_step_scanning_covers_nonterminating_runtime_boundaries() {
    let unresolved_directory = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - working-directory: '${{ matrix.directory }}'\n    run: echo unresolved",
        BTreeSet::new(),
    );
    assert!(unresolved_directory.indeterminate);

    let tolerated_unresolved_directory = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    working-directory: '${{ matrix.directory }}'\n    run: echo unresolved",
        BTreeSet::new(),
    );
    assert!(
        !tolerated_unresolved_directory.failed && !tolerated_unresolved_directory.indeterminate
    );

    let tolerated_directory = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    working-directory: missing\n    run: echo absent",
        BTreeSet::new(),
    );
    assert!(!tolerated_directory.failed && !tolerated_directory.indeterminate);

    let uncertain_directory = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - if: github.event.unknown\n    working-directory: missing\n    run: echo absent",
        BTreeSet::new(),
    );
    assert!(uncertain_directory.indeterminate);

    let tolerated_unknown_shell = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    shell: fish\n    run: echo unknown",
        BTreeSet::new(),
    );
    assert!(!tolerated_unknown_shell.failed && !tolerated_unknown_shell.indeterminate);

    let unresolved_shell = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - shell: '${{ vars.SHELL }}'\n    run: exit 1",
        BTreeSet::new(),
    );
    assert!(unresolved_shell.indeterminate);

    let unsupported_implicit_shell = scan(
        "runs-on: windows-latest\nsteps:\n  - run: exit 1\n  - shell: bash\n    run: tsc --noEmit -p app/tsconfig.json",
        BTreeSet::new(),
    );
    assert!(unsupported_implicit_shell.indeterminate);
    assert!(unsupported_implicit_shell.projects.is_empty());

    let tolerated_implicit_shell = scan(
        "runs-on: windows-latest\nsteps:\n  - continue-on-error: true\n    run: exit 1",
        BTreeSet::new(),
    );
    assert!(!tolerated_implicit_shell.failed && !tolerated_implicit_shell.indeterminate);

    let tolerated_unsafe_body = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    run: eval true",
        BTreeSet::new(),
    );
    assert!(!tolerated_unsafe_body.failed && !tolerated_unsafe_body.indeterminate);
}

#[test]
fn tolerated_action_outcomes_and_local_docker_runners_remain_sound() {
    let tolerated = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - id: setup\n    continue-on-error: true\n    uses: actions/cache@v4\n  - if: steps.setup.outcome == 'success'\n    run: tsc --noEmit -p app/tsconfig.json",
        BTreeSet::new(),
    );
    assert!(tolerated.indeterminate && tolerated.projects.is_empty());

    let docker_actions = BTreeSet::from(["local-action".to_string()]);
    let windows = scan_with_catalog(
        "runs-on: windows-latest\nsteps:\n  - uses: actions/checkout@v4\n  - uses: ./local-action",
        super::super::super::local_actions::LocalActionCatalog::docker(docker_actions.clone()),
    );
    assert!(windows.failed);
    let linux = scan_with_catalog(
        "runs-on: ubuntu-latest\nsteps:\n  - uses: actions/checkout@v4\n  - uses: ./local-action",
        super::super::super::local_actions::LocalActionCatalog::docker(docker_actions),
    );
    assert!(!linux.failed && !linux.indeterminate);

    let sparse_checkout = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - uses: actions/checkout@v4\n    with: {sparse-checkout: ''}\n  - uses: ./local-action",
        BTreeSet::from(["local-action".to_string()]),
    );
    assert!(!sparse_checkout.failed && !sparse_checkout.indeterminate);
}

#[test]
fn run_steps_cover_empty_commands_dynamic_tolerance_and_static_success() {
    let dynamic_continue = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: '${{ inputs.tolerate }}'\n    run: echo hi",
        BTreeSet::new(),
    );
    let success = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - run: echo hi",
        BTreeSet::new(),
    );
    let tolerated_failure = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    run: exit 1",
        BTreeSet::new(),
    );
    assert!(!success.failed);
    assert!(
        !tolerated_failure.failed || dynamic_continue.indeterminate || !dynamic_continue.failed
    );

    let tolerated_unresolved_run = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    run: echo '${{ matrix.x }}'",
        BTreeSet::new(),
    );
    assert!(!tolerated_unresolved_run.failed && !tolerated_unresolved_run.indeterminate);

    let tolerated_unresolved_shell = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: true\n    shell: '${{ vars.SHELL }}'\n    run: echo hi",
        BTreeSet::new(),
    );
    assert!(!tolerated_unresolved_shell.failed && !tolerated_unresolved_shell.indeterminate);

    let nameless = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - working-directory: .\n    name: no-run",
        BTreeSet::new(),
    );
    assert!(!nameless.failed);

    let skipped = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - if: false\n    run: echo skip\n  - continue-on-error: true\n    uses: actions/checkout@v4\n  - run: echo hi",
        BTreeSet::new(),
    );
    assert!(!skipped.failed);

    let unknown_configuration = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - timeout-minutes: []\n    run: echo invalid",
        BTreeSet::new(),
    );
    assert!(unknown_configuration.failed || unknown_configuration.indeterminate);

    let unsafe_body = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - run: eval true",
        BTreeSet::new(),
    );
    assert!(unsafe_body.indeterminate);

    let unknown_shell = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - shell: fish\n    run: echo unknown",
        BTreeSet::new(),
    );
    assert!(unknown_shell.indeterminate);

    let invalid_condition = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - if: \"${{ fromJSON('[]') }}\"\n    run: echo hi",
        BTreeSet::new(),
    );
    assert!(invalid_condition.failed || invalid_condition.indeterminate);

    let uncertain_false_config = scan(
        "runs-on: ubuntu-latest\nsteps:\n  - if: github.event.unknown\n    timeout-minutes: []\n    run: echo hi",
        BTreeSet::new(),
    );
    assert!(uncertain_false_config.indeterminate || uncertain_false_config.failed);
}
