use super::{
    strategy_configuration_valid_for_inputs, strategy_shape_valid, string_field_valid,
    JOB_NAME_CONTEXTS, STEP_STRING_CONTEXTS,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    InputState, StaticValue,
};
use serde_yaml::Value;

fn strategy(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn string_fields_require_their_documented_expression_contexts() {
    let job = strategy("name: 'check ${{ matrix.target }}'");
    assert!(string_field_valid(
        job.as_mapping().unwrap(),
        "name",
        JOB_NAME_CONTEXTS,
        false
    ));

    let invalid_job = strategy("name: '${{ jobs.build.result }}'");
    assert!(!string_field_valid(
        invalid_job.as_mapping().unwrap(),
        "name",
        JOB_NAME_CONTEXTS,
        false
    ));

    let step = strategy(
        "name: \"${{ hashFiles('src/**') }}\"\nworking-directory: \"${{ hashFiles('src/**') }}\"",
    );
    assert!(string_field_valid(
        step.as_mapping().unwrap(),
        "name",
        STEP_STRING_CONTEXTS,
        true
    ));
    assert!(string_field_valid(
        step.as_mapping().unwrap(),
        "working-directory",
        STEP_STRING_CONTEXTS,
        true
    ));
    assert!(!string_field_valid(
        step.as_mapping().unwrap(),
        "working-directory",
        STEP_STRING_CONTEXTS,
        false
    ));
}

#[test]
fn strategy_fields_require_documented_contexts_and_scalar_shapes() {
    for yaml in [
        "fail-fast: false\nmax-parallel: 1",
        "fail-fast: '${{ github.event_name == ''push'' }}'\nmax-parallel: '${{ inputs.parallel }}'",
        "fail-fast: '${{ needs.setup.outputs.fail_fast }}'\nmax-parallel: '${{ vars.MAX_PARALLEL }}'",
    ] {
        assert!(strategy_shape_valid(Some(&strategy(yaml))), "{yaml}");
    }

    for yaml in [
        "fail-fast: 1",
        "fail-fast: 'false'",
        "max-parallel: 0",
        "max-parallel: -1",
        "max-parallel: 1.5",
        "max-parallel: '2'",
        "fail-fast: '${{ matrix.enabled }}'",
        "max-parallel: '${{ strategy.job-total }}'",
        "fail-fast: '${{ secrets.FAIL_FAST }}'",
        "max-parallel: '${{ env.PARALLEL }}'",
        "fail-fast: '${{ job.status == ''success'' }}'",
        "max-parallel: '${{ runner.os }}'",
        "fail-fast: '${{ steps.check.outcome == ''success'' }}'",
        "max-parallel: '${{ success() }}'",
        "fail-fast: \"${{ format('{0}', github.ref) }}\"",
        "fail-fast: '${{ 1 }}'",
        "fail-fast: \"${{ fromJSON('\\\"true\\\"') }}\"",
        "fail-fast: \"${{ fromJSON('5') }}\"",
        "max-parallel: \"${{ contains(github.ref, 'main') }}\"",
        "max-parallel: '${{ true }}'",
        "max-parallel: '${{ -1 }}'",
        "max-parallel: '${{ 1.5 }}'",
    ] {
        assert!(!strategy_shape_valid(Some(&strategy(yaml))), "{yaml}");
    }
}

#[test]
fn max_parallel_rechecks_resolved_input_values() {
    let job = strategy("strategy:\n  max-parallel: '${{ inputs.parallel }}'");
    let mut inputs = InputState::new();
    inputs.insert("parallel".to_string(), StaticValue::Number("0".to_string()));
    assert!(!strategy_configuration_valid_for_inputs(&job, &inputs));

    inputs.insert("parallel".to_string(), StaticValue::Number("2".to_string()));
    assert!(strategy_configuration_valid_for_inputs(&job, &inputs));

    let dynamic = strategy("strategy:\n  max-parallel: '${{ github.run_number }}'");
    assert!(strategy_configuration_valid_for_inputs(
        &dynamic,
        &InputState::new()
    ));
}
