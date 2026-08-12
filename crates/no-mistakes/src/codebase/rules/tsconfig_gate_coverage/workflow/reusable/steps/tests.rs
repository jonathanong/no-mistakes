use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{direct_inputs, inputs_with_matrix_values, MatrixState},
    reusable::model::{GithubEventContext, GithubRef},
};
use crate::codebase::{
    ci_graph::parse::parse_workflow_value, rules::tsconfig_gate_coverage::ProjectSourceInputs,
};

fn scan(job: &str, local_actions: BTreeSet<String>) -> StepScan {
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
    scan_job_steps(
        &job,
        &triggers,
        &inputs,
        &EnvironmentState::default(),
        None,
        None,
        &context,
    )
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
}
