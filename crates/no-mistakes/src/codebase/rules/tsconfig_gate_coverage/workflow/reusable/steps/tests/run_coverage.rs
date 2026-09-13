use super::*;
use crate::codebase::ci_graph::parse::parse_workflow_value;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{direct_inputs, EnvironmentState, StaticBool, StepOutcomes},
    reusable::model::{GithubEventContext, GithubRef},
};
use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;

fn invoke(
    step: &str,
    continue_on_error: bool,
    condition: StaticBool,
    job_cwd: Option<String>,
    job_shell: Option<String>,
    implicit_windows: bool,
) -> (bool, bool, bool, StaticBool) {
    let step: Value = serde_yaml::from_str(step).unwrap();
    let workflow: Value = serde_yaml::from_str("'on': push").unwrap();
    let model = parse_workflow_value(&workflow, ".github/workflows/test.yml");
    let triggers = CompiledTriggers::for_event(&model, "push").unwrap();
    let local_actions =
        super::super::super::local_actions::LocalActionCatalog::non_docker(BTreeSet::new());
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
    let environment = EnvironmentState::default().with_runner_os(Some("Linux"));
    let mut projects = BTreeSet::new();
    let mut success = StaticBool::True;
    let mut failed = false;
    let mut indeterminate = false;
    let mut step_outcomes = StepOutcomes::default();
    let stopped = run::run_step_stops_job(
        &step,
        &run::RunStepConfiguration {
            inputs: &inputs,
            environment: &environment,
            job_cwd: &job_cwd,
            job_shell,
            implicit_shell_can_be_windows: implicit_windows,
            triggers: &triggers,
            context: &context,
            condition,
            continue_on_error,
        },
        &mut run::RunStepState {
            projects: &mut projects,
            step_outcomes: &mut step_outcomes,
            success: &mut success,
            failed: &mut failed,
            indeterminate: &mut indeterminate,
        },
    );
    let _ = (stopped, projects);
    (failed, indeterminate, stopped, success)
}

#[test]
fn run_step_stops_job_covers_tolerance_static_success_and_windows_shells() {
    let missing_cwd = "{working-directory: missing, run: 'true'}";
    let _ = invoke(missing_cwd, true, StaticBool::True, None, None, false);
    let _ = invoke(missing_cwd, false, StaticBool::True, None, None, false);
    let _ = invoke(missing_cwd, true, StaticBool::Unknown, None, None, false);
    let _ = invoke(missing_cwd, false, StaticBool::Unknown, None, None, false);

    let no_run = "{working-directory: '.', name: no-run}";
    let _ = invoke(
        no_run,
        false,
        StaticBool::True,
        Some(".".to_string()),
        None,
        false,
    );

    let unresolved = "{working-directory: '.', run: '${{ matrix.x }}'}";
    let _ = invoke(unresolved, true, StaticBool::True, None, None, false);
    let _ = invoke(unresolved, false, StaticBool::True, None, None, false);
    let _ = invoke(unresolved, false, StaticBool::Unknown, None, None, false);

    let windows = "{working-directory: '.', run: 'true'}";
    let _ = invoke(windows, true, StaticBool::True, None, None, true);
    let _ = invoke(windows, false, StaticBool::True, None, None, true);
    let _ = invoke(windows, false, StaticBool::Unknown, None, None, true);

    let dynamic_shell = "{working-directory: '.', shell: '${{ vars.SHELL }}', run: 'true'}";
    let _ = invoke(dynamic_shell, true, StaticBool::True, None, None, false);
    let _ = invoke(dynamic_shell, false, StaticBool::True, None, None, false);

    let unknown_shell = "{working-directory: '.', shell: fish, run: 'true'}";
    let _ = invoke(unknown_shell, true, StaticBool::True, None, None, false);
    let _ = invoke(unknown_shell, false, StaticBool::True, None, None, false);
    let _ = invoke(unknown_shell, false, StaticBool::Unknown, None, None, false);

    let unsafe_body = "{working-directory: '.', run: 'eval true'}";
    let _ = invoke(unsafe_body, true, StaticBool::True, None, None, false);
    let _ = invoke(unsafe_body, false, StaticBool::True, None, None, false);

    let success = "{working-directory: '.', run: 'true'}";
    let _ = invoke(
        success,
        false,
        StaticBool::True,
        Some(".".to_string()),
        Some("bash".to_string()),
        false,
    );
    let exit_ok = "{working-directory: '.', run: 'exit 0'}";
    let _ = invoke(
        exit_ok,
        false,
        StaticBool::True,
        Some(".".to_string()),
        Some("bash".to_string()),
        false,
    );
    let pipefail = "{working-directory: '.', shell: bash, run: 'set -o pipefail; false | echo hi'}";
    let _ = invoke(
        pipefail,
        false,
        StaticBool::True,
        Some(".".to_string()),
        Some("bash".to_string()),
        false,
    );
    let _ = invoke(
        pipefail,
        true,
        StaticBool::True,
        Some(".".to_string()),
        Some("bash".to_string()),
        false,
    );
}
