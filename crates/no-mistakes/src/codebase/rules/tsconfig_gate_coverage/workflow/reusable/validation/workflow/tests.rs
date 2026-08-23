use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    inputs_with_matrix_values, InputState, MatrixState, StaticValue,
};
use std::collections::BTreeMap;

fn workflow(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn workflow_shape_requires_known_top_level_keys_and_supported_field_shapes() {
    assert!(workflow_shape_valid(&workflow(
        "name: checks\nrun-name: 'checks-${{ github.ref }}'\non: push\npermissions: read-all\nenv:\n  NODE_ENV: 'test-${{ github.ref }}'\ndefaults:\n  run:\n    shell: bash\n    working-directory: app\nconcurrency:\n  group: 'checks-${{ github.ref }}'\n  cancel-in-progress: true\njobs: {}",
    )));
    assert!(workflow_shape_valid(&workflow(
        "on: push\nenv: {}\njobs: {}"
    )));
    assert!(workflow_shape_valid(&workflow(
        "on: push\nenv: {ENABLED: true, RETRIES: 2}\nconcurrency:\n  group: checks\n  cancel-in-progress: '${{ inputs.cancel }}'\njobs: {}"
    )));
    for run_name in ["''", "'   '"] {
        assert!(workflow_shape_valid(&workflow(&format!(
            "on: push\nrun-name: {run_name}\njobs: {{}}"
        ))));
    }

    for yaml in [
        "on: push\nbogus: true\njobs: {}",
        "on: push\nname: [checks]\njobs: {}",
        "on: push\nrun-name: '${{ secrets.TOKEN }}'\njobs: {}",
        "on: push\nenv: []\njobs: {}",
        "on: push\nenv:\n  1: value\njobs: {}",
        "on: push\nenv:\n  BROKEN: null\njobs: {}",
        "on: push\nenv:\n  BROKEN: '${{ }}'\njobs: {}",
        "on: push\ndefaults: []\njobs: {}",
        "on: push\ndefaults:\n  run: []\njobs: {}",
        "on: push\ndefaults:\n  run: {}\njobs: {}",
        "on: push\ndefaults:\n  run:\n    shell: ''\njobs: {}",
        "on: push\ndefaults:\n  run:\n    shell: '${{ }}'\njobs: {}",
        "on: push\ndefaults:\n  run:\n    working-directory: ''\njobs: {}",
        "on: push\ndefaults:\n  run:\n    bogus: true\njobs: {}",
        "on: push\nconcurrency: []\njobs: {}",
        "on: push\nconcurrency: ''\njobs: {}",
        "on: push\nconcurrency: 'checks-${{ }}'\njobs: {}",
        "on: push\nconcurrency:\n  group: [checks]\njobs: {}",
        "on: push\nconcurrency:\n  group: ''\njobs: {}",
        "on: push\nconcurrency:\n  group: 'checks-${{ }}'\njobs: {}",
        "on: push\nconcurrency:\n  group: checks\n  cancel-in-progress: invalid\njobs: {}",
    ] {
        assert!(!workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
}

#[test]
fn workflow_env_uses_workflow_level_contexts_and_functions() {
    for yaml in [
        "on: push\nenv:\n  REF: '${{ github.ref }}'\n  TOKEN: '${{ secrets.TOKEN }}'\n  INPUT: '${{ inputs.target }}'\n  VARIABLE: '${{ vars.ENVIRONMENT }}'\njobs: {}",
        "on: push\nenv:\n  FORMATTED: \"${{ format('{0}', github.ref_name) }}\"\njobs: {}",
    ] {
        assert!(workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
    for yaml in [
        "on: push\nenv:\n  JOB: '${{ jobs.typecheck.outputs.version }}'\njobs: {}",
        "on: push\nenv:\n  NEED: '${{ needs.setup.outputs.version }}'\njobs: {}",
        "on: push\nenv:\n  HASH: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\njobs: {}",
        "on: push\nenv:\n  STATUS: '${{ success() }}'\njobs: {}",
    ] {
        assert!(!workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
}

#[test]
fn workflow_defaults_and_concurrency_follow_workflow_context_rules() {
    for yaml in [
        "defaults:\n  run:\n    shell: bash\n    working-directory: packages/app",
        "concurrency: checks-${{ github.ref }}",
        "concurrency:\n  group: checks\n  queue: max",
        "concurrency:\n  group: checks-${{ vars.ENVIRONMENT }}\n  cancel-in-progress: '${{ inputs.cancel }}'",
        "concurrency:\n  group: checks\n  cancel-in-progress: \"${{ fromJSON('false') }}\"",
    ] {
        let value = workflow(yaml);
        assert!(
            workflow_shape_valid(&value),
            "workflow-level value should be valid: {yaml}"
        );
    }

    for yaml in [
        "defaults:\n  run:\n    shell: '${{ github.ref }}'",
        "defaults:\n  run:\n    working-directory: 'packages/${{ inputs.package }}'",
        "concurrency: checks-${{ needs.setup.outputs.key }}",
        "concurrency:\n  group: checks-${{ matrix.package }}",
        "concurrency:\n  group: checks\n  cancel-in-progress: '${{ secrets.CANCEL }}'",
        "concurrency:\n  group: checks\n  cancel-in-progress: \"${{ 'false' }}\"",
        "concurrency:\n  group: checks\n  cancel-in-progress: \"${{ fromJSON('\\\"false\\\"') }}\"",
        "concurrency:\n  group: checks\n  queue: min",
        "concurrency:\n  group: checks\n  queue: '${{ inputs.queue }}'",
        "concurrency:\n  group: checks\n  queue: 100",
    ] {
        let value = workflow(yaml);
        assert!(
            !workflow_shape_valid(&value),
            "workflow-level value should be invalid: {yaml}"
        );
    }
}

#[test]
fn job_defaults_and_concurrency_follow_job_context_rules() {
    for yaml in [
        "defaults:\n  run:\n    shell: bash\n    working-directory: packages/app",
        "defaults:\n  run:\n    shell: 'bash ${{ github.ref }}'\n    working-directory: 'packages/${{ needs.setup.outputs.package }}'",
        "defaults:\n  run:\n    shell: 'bash ${{ strategy.job-index }}'\n    working-directory: 'packages/${{ matrix.package }}'",
        "defaults:\n  run:\n    shell: 'bash ${{ vars.SHELL_FLAGS }}'\n    working-directory: 'packages/${{ inputs.package }}'",
        "defaults:\n  run:\n    shell: 'bash ${{ env.SHELL }}'",
        "concurrency: checks-${{ github.ref }}",
        "concurrency:\n  group: checks\n  queue: max",
        "concurrency:\n  group: checks-${{ needs.setup.outputs.key }}\n  cancel-in-progress: '${{ matrix.cancel }}'",
        "concurrency:\n  group: checks-${{ strategy.job-index }}\n  cancel-in-progress: '${{ inputs.cancel }}'",
        "concurrency: checks-${{ vars.ENVIRONMENT }}",
        "concurrency:\n  group: checks\n  cancel-in-progress: \"${{ fromJSON('false') }}\"",
    ] {
        let value = workflow(yaml);
        assert!(
            job_defaults_shape_valid(value.get("defaults"))
                && job_concurrency_shape_valid(value.get("concurrency")),
            "job-level value should be valid: {yaml}"
        );
    }

    for yaml in [
        "defaults:\n  run:\n    working-directory: 'packages/${{ secrets.PACKAGE }}'",
        "concurrency: checks-${{ env.CI }}",
        "concurrency:\n  group: checks\n  cancel-in-progress: '${{ secrets.CANCEL }}'",
        "concurrency:\n  group: checks\n  cancel-in-progress: \"${{ 'false' }}\"",
        "concurrency:\n  group: checks\n  cancel-in-progress: \"${{ fromJSON('\\\"false\\\"') }}\"",
        "concurrency:\n  group: checks\n  queue: min",
        "concurrency:\n  group: checks\n  queue: '${{ matrix.queue }}'",
        "concurrency:\n  group: checks\n  queue: true",
    ] {
        let value = workflow(yaml);
        assert!(
            !job_defaults_shape_valid(value.get("defaults"))
                || !job_concurrency_shape_valid(value.get("concurrency")),
            "job-level value should be invalid: {yaml}"
        );
    }
}

#[test]
fn concurrency_rejects_statically_empty_context_free_groups() {
    for yaml in [
        "concurrency: '${{ '' }}'",
        "concurrency:\n  group: '${{ '' }}'",
        "concurrency: '${{ true }}'",
        "concurrency:\n  group: '${{ true }}'",
    ] {
        let value = workflow(yaml);
        assert!(
            !workflow_concurrency_shape_valid(value.get("concurrency")),
            "workflow-level value should reject an empty group: {yaml}"
        );
        assert!(
            !job_concurrency_shape_valid(value.get("concurrency")),
            "job-level value should reject an empty group: {yaml}"
        );
    }

    for yaml in [
        "concurrency: \"${{ 'checks' }}\"",
        "concurrency: checks-${{ github.ref }}",
        "concurrency:\n  group: checks-${{ matrix.package }}",
    ] {
        let value = workflow(yaml);
        assert!(
            job_concurrency_shape_valid(value.get("concurrency")),
            "job-level value should preserve dynamic groups: {yaml}"
        );
    }
}

#[test]
fn concurrency_groups_recheck_resolved_activation_values() {
    let input_group = workflow("concurrency: '${{ inputs.group }}'");
    let embedded_input_group = workflow("concurrency: 'checks-${{ fromJSON(inputs.group) }}'");
    let matrix_group = workflow("concurrency: '${{ matrix.group }}'");
    let input_group = input_group.get("concurrency");
    let embedded_input_group = embedded_input_group.get("concurrency");
    let matrix_group = matrix_group.get("concurrency");
    let mut inputs = InputState::new();

    inputs.insert(
        "group".to_string(),
        StaticValue::String("checks".to_string()),
    );
    assert!(workflow_concurrency_valid_for_inputs(input_group, &inputs));
    assert!(job_concurrency_valid_for_inputs(input_group, &inputs));

    for value in [StaticValue::Bool(false), StaticValue::Number("42".into())] {
        inputs.insert("group".to_string(), value);
        assert!(workflow_concurrency_valid_for_inputs(input_group, &inputs));
        assert!(job_concurrency_valid_for_inputs(input_group, &inputs));
    }

    inputs.insert("group".to_string(), StaticValue::String(String::new()));
    assert!(!workflow_concurrency_valid_for_inputs(input_group, &inputs));
    assert!(!job_concurrency_valid_for_inputs(input_group, &inputs));

    inputs.insert("group".to_string(), StaticValue::NonStringable);
    assert!(!workflow_concurrency_valid_for_inputs(input_group, &inputs));
    assert!(!job_concurrency_valid_for_inputs(input_group, &inputs));

    inputs.insert("group".to_string(), StaticValue::Mapping);
    assert!(!workflow_concurrency_valid_for_inputs(input_group, &inputs));
    assert!(!job_concurrency_valid_for_inputs(input_group, &inputs));

    inputs.insert("group".to_string(), StaticValue::Unknown);
    assert!(workflow_concurrency_valid_for_inputs(input_group, &inputs));
    assert!(job_concurrency_valid_for_inputs(input_group, &inputs));

    inputs.insert("group".to_string(), StaticValue::String("{}".to_string()));
    assert!(!workflow_concurrency_valid_for_inputs(
        embedded_input_group,
        &inputs
    ));
    assert!(!job_concurrency_valid_for_inputs(
        embedded_input_group,
        &inputs
    ));

    let matrix_inputs = inputs_with_matrix_values(
        &InputState::new(),
        &BTreeMap::from([(String::from("group"), Value::String(String::new()))]),
        MatrixState::Static,
    );
    assert!(!job_concurrency_valid_for_inputs(
        matrix_group,
        &matrix_inputs
    ));
}

#[test]
fn concurrency_cancel_in_progress_rechecks_resolved_boolean_values() {
    let input_cancel =
        workflow("concurrency:\n  group: checks\n  cancel-in-progress: '${{ inputs.cancel }}'");
    let matrix_cancel =
        workflow("concurrency:\n  group: checks\n  cancel-in-progress: '${{ matrix.cancel }}'");
    let input_cancel = input_cancel.get("concurrency");
    let matrix_cancel = matrix_cancel.get("concurrency");
    let mut inputs = InputState::new();

    inputs.insert("cancel".to_string(), StaticValue::Bool(true));
    assert!(job_concurrency_valid_for_inputs(input_cancel, &inputs));

    inputs.insert(
        "cancel".to_string(),
        StaticValue::String("true".to_string()),
    );
    assert!(!job_concurrency_valid_for_inputs(input_cancel, &inputs));

    inputs.insert("cancel".to_string(), StaticValue::Number("1".to_string()));
    assert!(!job_concurrency_valid_for_inputs(input_cancel, &inputs));

    inputs.insert("cancel".to_string(), StaticValue::Unknown);
    assert!(job_concurrency_valid_for_inputs(input_cancel, &inputs));

    let boolean_matrix = inputs_with_matrix_values(
        &InputState::new(),
        &BTreeMap::from([(String::from("cancel"), Value::Bool(false))]),
        MatrixState::Static,
    );
    assert!(job_concurrency_valid_for_inputs(
        matrix_cancel,
        &boolean_matrix
    ));

    let string_matrix = inputs_with_matrix_values(
        &InputState::new(),
        &BTreeMap::from([(String::from("cancel"), Value::String("false".to_string()))]),
        MatrixState::Static,
    );
    assert!(!job_concurrency_valid_for_inputs(
        matrix_cancel,
        &string_matrix
    ));
}

#[test]
fn permissions_follow_the_actions_scope_and_access_schema() {
    assert!(!permission_value_valid(&Value::Number(1.into()), "read"));
    for yaml in [
        "on: push\npermissions: read-all\njobs: {}",
        "on: push\npermissions: write-all\njobs: {}",
        "on: push\npermissions: {}\njobs: {}",
        "on: push\npermissions:\n  contents: read\n  id-token: write\n  models: read\n  code-quality: write\n  vulnerability-alerts: read\njobs: {}",
    ] {
        assert!(workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
    for yaml in [
        "on: push\npermissions: bogus\njobs: {}",
        "on: push\npermissions: none\njobs: {}",
        "on: push\npermissions: []\njobs: {}",
        "on: push\npermissions:\n  bogus: read\njobs: {}",
        "on: push\npermissions:\n  1: read\njobs: {}",
        "on: push\npermissions:\n  contents: invalid\njobs: {}",
        "on: push\npermissions:\n  id-token: read\njobs: {}",
        "on: push\npermissions:\n  models: write\njobs: {}",
        "on: push\npermissions:\n  code-quality: invalid\njobs: {}",
        "on: push\npermissions:\n  vulnerability-alerts: write\njobs: {}",
    ] {
        assert!(!workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
}
