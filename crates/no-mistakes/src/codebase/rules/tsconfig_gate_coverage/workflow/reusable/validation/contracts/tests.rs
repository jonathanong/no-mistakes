use super::*;

mod defaults;

fn on(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn contract_shape_validates_every_declaration_kind() {
    assert!(!workflow_call_shape_valid(Some(&Value::Bool(true))));
    assert!(!workflow_call_trigger_keys_valid(&Value::Bool(true)));
    assert!(workflow_call_shape_valid(Some(&on("push"))));
    assert!(workflow_call_shape_valid(Some(&on("image_version"))));
    assert!(workflow_call_shape_valid(Some(&on(
        "[push, workflow_call]"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on("pussh"))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "[push, workflow_dispath]"
    ))));
    for webhook_only in [
        "project",
        "project_card",
        "project_column",
        "repository",
        "repository_import",
        "repository_vulnerability_alert",
        "secret_scanning_alert",
        "team_add",
        "workflow_job",
    ] {
        assert!(
            !workflow_call_shape_valid(Some(&on(&format!("push:\n{webhook_only}:")))),
            "{webhook_only}"
        );
    }
    assert!(!workflow_call_shape_valid(Some(&on(
        "push:\nworkflow_dispath:"
    ))));
    for malformed in ["push: []", "push: true", "push: invalid"] {
        assert!(
            !workflow_call_shape_valid(Some(&on(malformed))),
            "{malformed}"
        );
    }
    assert!(workflow_call_shape_valid(Some(&on(
        "push:\nschedule:\n  - cron: '0 0 * * *'"
    ))));
    for malformed in [
        "schedule:",
        "schedule: []",
        "schedule:\n  - {}",
        "schedule:\n  - cron: true",
        "schedule:\n  - cron: ''",
        "schedule:\n  - cron: '0 0 * * *'\n    unknown: true",
    ] {
        assert!(
            !workflow_call_shape_valid(Some(&on(malformed))),
            "{malformed}"
        );
    }
    assert!(workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      value: '${{ jobs.build.outputs.result }}'\n      description: result"
    ))));
    assert!(workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      value: 'result-${{ jobs.build.outputs.result }}'"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      value: [invalid]"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      value: 'result-${{ }}'"
    ))));
    for unavailable in ["secrets.TOKEN", "steps.build.outputs.result"] {
        assert!(!workflow_call_shape_valid(Some(&on(&format!(
            "workflow_call:\n  outputs:\n    result:\n      value: '${{{{ {unavailable} }}}}'"
        )))));
    }
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  secrets:\n    token:\n      required: yes"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  inputs:\n    enabled:\n      type: boolean\n      unknown: true"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  inputs:\n    enabled:\n      description: missing type"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  inputs:\n    enabled:\n      type: invalid"
    ))));
    assert!(!workflow_call_shape_valid(Some(&on(
        "workflow_call:\n  outputs:\n    result:\n      description: missing value"
    ))));
}

#[test]
fn trigger_configs_reject_values_actions_cannot_schedule() {
    for valid in [
        "push:\n  branches: [main]\n  paths: ['src/**']",
        "push:\n  branches: [main, '!release/**']\n  tags: ['v*', '!v0.*']\n  paths: ['src/**', '!src/generated/**']",
        "pull_request:\n  types: [opened]\n  branches-ignore: [release/**]",
        "merge_group:\n  types: [checks_requested]",
        "workflow_run:\n  workflows: [CI]\n  types: [completed]",
        "repository_dispatch:\n  types: [refresh]",
        "issues:\n  types: [opened]",
        "check_suite:\n  types: [completed]",
        "check_suite:\n  types: [requested]",
        "check_suite:\n  types: [rerequested]",
        "image_version:\n  names: [app]\n  versions: ['1.*']",
        "schedule:\n  - cron: '*/15 0-23 1,15 JAN-MAR MON-FRI'",
        "workflow_dispatch:\n  inputs:\n    deploy:\n      type: boolean\n      required: true",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: [staging, production]",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: [staging, production]\n      default: staging",
        "workflow_dispatch:\n  inputs:\n    deploy:\n      type: boolean\n      default: false\n    count:\n      type: number\n      default: 2\n    label:\n      type: string\n      default: release\n    target:\n      type: environment\n      default: production",
        "workflow_dispatch: {}",
    ] {
        assert!(workflow_call_shape_valid(Some(&on(valid))), "{valid}");
    }

    for malformed in [
        "push: []",
        "schedule",
        "[push, schedule]",
        "workflow_run",
        "[push, workflow_run]",
        "workflow_run:",
        "push:\n  branches: []",
        "push:\n  branches: ['   ']",
        "push:\n  branches: ['${{ github.ref }}']",
        "push:\n  paths: ['src/${{ matrix.target }}/**']",
        "push:\n  branches: [main]\n  branches-ignore: [release/**]",
        "push:\n  paths: ['src/**']\n  paths-ignore: ['src/generated/**']",
        "push:\n  branches: ['!release/**']",
        "push:\n  tags: ['!v0.*']",
        "push:\n  paths: ['!src/generated/**']",
        "pull_request:\n  branches: ['!release/**']",
        "pull_request:\n  paths: ['!src/generated/**']",
        "workflow_run:\n  workflows: [CI]\n  branches: ['!release/**']",
        "merge_group:\n  branches: ['!release/**']",
        "push:\n  unknown: true",
        "pull_request:\n  tags: [v*]",
        "workflow_run:\n  types: [completed]",
        "workflow_run:\n  workflows: CI",
        "workflow_run:\n  workflows: ['${{ github.workflow }}']",
        "repository_dispatch:\n  types: refresh",
        "repository_dispatch:\n  types: ['${{ github.event.action }}']",
        "issues:\n  unknown: true",
        "issues:\n  types: [not_an_issue_event]",
        "check_suite:\n  types: [not_a_check_suite_event]",
        "pull_request:\n  types: [not_a_pull_request_event]",
        "merge_group:\n  types: [completed]",
        "workflow_run:\n  workflows: [CI]\n  types: [not_a_workflow_run_event]",
        "image_version:\n  types: [created]",
        "create:\n  types: [created]",
        "workflow_dispatch:\n  unknown: true",
        "workflow_dispatch:\n  inputs:\n    environment: true",
        "workflow_dispatch:\n  inputs:\n    'bad name':\n      type: string",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: string\n      options: [staging]",
        "workflow_dispatch:\n  inputs:\n    deploy:\n      type: boolean\n      default: text",
        "workflow_dispatch:\n  inputs:\n    count:\n      type: number\n      default: text",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: [staging]\n      default: production",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: ['', production]",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: [staging, 1]",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: [staging, production]\n      default: true",
        "workflow_dispatch:\n  inputs:\n    environment:\n      type: choice\n      options: [staging, production]\n      default: 1",
        "schedule:\n  - cron: nope",
        "schedule:\n  - cron: '0 0 * *'",
        "schedule:\n  - cron: '60 0 * * *'",
        "schedule:\n  - cron: '0 0 0 * *'",
        "schedule:\n  - cron: '0 0 * 0 *'",
        "schedule:\n  - cron: '0 0 * * */0'",
    ] {
        assert!(
            !workflow_call_shape_valid(Some(&on(malformed))),
            "{malformed}"
        );
    }
    let twenty_five_inputs = (0..25)
        .map(|index| format!("    input-{index}:\n      type: string"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(workflow_call_shape_valid(Some(&on(&format!(
        "workflow_dispatch:\n  inputs:\n{twenty_five_inputs}"
    )))));
    let twenty_six_inputs = format!("{twenty_five_inputs}\n    input-25:\n      type: string");
    assert!(!workflow_call_shape_valid(Some(&on(&format!(
        "workflow_dispatch:\n  inputs:\n{twenty_six_inputs}"
    )))));
}
