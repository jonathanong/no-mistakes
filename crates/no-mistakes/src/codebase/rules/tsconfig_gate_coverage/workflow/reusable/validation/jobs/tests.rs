use super::*;
use serde_yaml::Value;
use std::collections::BTreeMap;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    inputs_with_matrix_values, InputState, MatrixState, StaticValue,
};

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn steps_cannot_mix_action_and_shell_commands() {
    assert!(!steps_shape_valid(&job("runs-on: ubuntu-latest")));
    assert!(steps_shape_valid(&job(
        "uses: owner/repository/.github/workflows/checks.yml@main"
    )));
    assert!(steps_shape_valid(&job("steps:\n  - run: echo ok")));
    assert!(steps_shape_valid(&job("steps:\n  - uses: owner/action@v1")));
    for yaml in [
        "steps: invalid",
        "steps: []",
        "steps:\n  - name: inert",
        "steps:\n  - run: ''",
        "steps:\n  - run: []",
        "steps:\n  - uses: true",
        "steps:\n  - run: echo no\n    uses: owner/action@v1",
    ] {
        assert!(!steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn steps_require_known_keys_and_matching_value_shapes() {
    for yaml in [
        "steps:\n  - name: run\n    id: run\n    if: true\n    run: echo ok\n    working-directory: app\n    shell: bash\n    env: {NODE_ENV: test}\n    continue-on-error: false\n    timeout-minutes: 5",
        "steps:\n  - name: action\n    id: action\n    if: '${{ always() }}'\n    uses: actions/checkout@v4\n    with: {fetch-depth: 0}\n    env: {NODE_ENV: test}\n    continue-on-error: '${{ false }}'\n    timeout-minutes: '${{ inputs.timeout }}'",
        "steps:\n  - name: action ${{ github.ref }}\n    uses: actions/checkout@v4\n    with: {ref: 'refs/${{ github.ref_name }}'}\n    env: {NODE_ENV: '${{ github.ref_name }}'}",
        "steps:\n  - if: env.RUN && runner.os && steps.setup.outputs.enabled\n    run: echo valid",
        "steps:\n  - if: hashFiles('**/pnpm-lock.yaml') != ''\n    run: echo valid",
        "steps:\n  - run: \"echo ${{ github.ref }} ${{ secrets.TOKEN }} ${{ hashFiles('**/pnpm-lock.yaml') }}\"",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - run: echo invalid\n    bogus: true",
        "steps:\n  - name: false\n    run: echo invalid",
        "steps:\n  - if: []\n    run: echo invalid",
        "steps:\n  - if: '${{ }}'\n    run: echo invalid",
        "steps:\n  - if: 'true &&'\n    run: echo invalid",
        "steps:\n  - if: '${{ contains() }}'\n    run: echo invalid",
        "steps:\n  - if: '${{ always(1) }}'\n    run: echo invalid",
        "steps:\n  - if: '${{ hashFiles() }}'\n    run: echo invalid",
        "steps:\n  - if: secrets.TYPECHECK\n    run: echo invalid",
        "steps:\n  - run: 'echo ${{ }}'",
        "steps:\n  - run: 'tsc --noEmit ${{ jobs.typecheck.outputs.project }}'",
        "steps:\n  - run: 'tsc --noEmit ${{ success() }}'",
        "steps:\n  - uses: actions/checkout@v4\n    with: {ref: '${{ }}'}",
        "steps:\n  - run: echo invalid\n    working-directory: true",
        "steps:\n  - run: echo invalid\n    shell: true",
        "steps:\n  - run: echo invalid\n    env: [invalid]",
        "steps:\n  - run: echo invalid\n    continue-on-error: []",
        "steps:\n  - run: echo invalid\n    timeout-minutes: five",
        "steps:\n  - run: echo invalid\n    timeout-minutes: 0",
        "steps:\n  - run: echo invalid\n    timeout-minutes: 1.5",
        "steps:\n  - run: echo invalid\n    timeout-minutes: 361",
        "steps:\n  - uses: actions/checkout@v4\n    with: true",
        "steps:\n  - uses: actions/checkout@v4\n    shell: bash",
    ] {
        assert!(!steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn job_and_step_environment_expressions_use_distinct_documented_contexts() {
    let valid = job(
        "runs-on: ubuntu-latest\nenv:\n  REF: '${{ needs.setup.outputs.ref }}'\n  TOKEN: '${{ secrets.TOKEN }}'\nsteps:\n  - env:\n      JOB: '${{ job.status }}'\n      RUNNER: '${{ runner.os }}'\n      PRIOR: '${{ steps.setup.outputs.value }}'\n      OUTER: '${{ env.OUTER }}'\n      FILES: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\n    run: echo valid",
    );
    assert!(super::step_job_shape_valid(&valid));
    assert!(super::steps_shape_valid(&valid));

    for yaml in [
        "runs-on: ubuntu-latest\nenv: {RESULT: '${{ env.OUTER }}'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nenv: {RESULT: '${{ jobs.build.outputs.result }}'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nenv: {RESULT: '${{ steps.setup.outputs.result }}'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nenv: {RESULT: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"}\nsteps:\n  - run: echo invalid",
    ] {
        assert!(!super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "runs-on: ubuntu-latest\nsteps:\n  - env: {RESULT: '${{ jobs.build.outputs.result }}'}\n    run: echo invalid",
        "runs-on: ubuntu-latest\nsteps:\n  - env: {RESULT: '${{ success() }}'}\n    run: echo invalid",
    ] {
        assert!(!super::steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn job_outputs_reject_the_reusable_workflow_jobs_context() {
    assert!(super::step_job_shape_valid(&job(
        "runs-on: ubuntu-latest\noutputs: {}\nsteps:\n  - run: echo valid"
    )));
    for yaml in [
        "runs-on: ubuntu-latest\noutputs: {result: '${{ steps.collect.outputs.result }}'}\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\noutputs: {result: '${{ needs.prepare.outputs.result }}'}\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\noutputs: {result: '${{ secrets.TOKEN }}'}\nsteps:\n  - run: echo valid",
    ] {
        assert!(super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
    assert!(!super::step_job_shape_valid(&job(
        "runs-on: ubuntu-latest\noutputs: {result: '${{ jobs.build.outputs.result }}'}\nsteps:\n  - run: echo invalid"
    )));
}

#[test]
fn step_ids_must_be_unique_case_insensitive_identifiers() {
    for yaml in [
        "steps:\n  - id: build\n    run: cargo build\n  - id: test\n    run: cargo test",
        "steps:\n  - id: Build_1-check\n    run: cargo test",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - id: build\n    run: cargo build\n  - id: build\n    run: cargo test",
        "steps:\n  - id: Build\n    run: cargo build\n  - id: build\n    run: cargo test",
        "steps:\n  - id: 1build\n    run: cargo build",
        "steps:\n  - id: build step\n    run: cargo build",
    ] {
        assert!(!steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn hash_files_is_available_only_in_step_conditions() {
    assert!(!super::step_job_shape_valid(&job(
        "if: hashFiles('**/pnpm-lock.yaml') != ''\nruns-on: ubuntu-latest\nsteps:\n  - run: echo invalid"
    )));
    assert!(super::step_job_shape_valid(&job(
        "runs-on: ubuntu-latest\nsteps:\n  - if: hashFiles('**/pnpm-lock.yaml') != ''\n    run: echo valid"
    )));
    assert!(!super::step_job_shape_valid(&job(
        "if: '${{ matrix.enabled }}'\nruns-on: ubuntu-latest\nstrategy:\n  matrix:\n    enabled: [true]\nsteps:\n  - run: echo invalid"
    )));
    assert!(super::step_job_shape_valid(&job(
        "continue-on-error: '${{ matrix.enabled }}'\nruns-on: ubuntu-latest\nstrategy:\n  matrix:\n    enabled: [true]\nsteps:\n  - run: echo valid"
    )));
}

#[test]
fn runs_on_uses_its_documented_contexts_without_expression_functions() {
    for yaml in [
        "runs-on: '${{ matrix.runner }}'\nsteps:\n  - run: echo valid",
        "runs-on: ['self-hosted', '${{ vars.RUNNER_LABEL }}']\nsteps:\n  - run: echo valid",
        "runs-on: \"${{ format('{0}', matrix.runner) }}\"\nsteps:\n  - run: echo valid",
    ] {
        assert!(super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }

    for yaml in [
        "runs-on: '${{ secrets.RUNNER }}'\nsteps:\n  - run: echo invalid",
        "runs-on: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\nsteps:\n  - run: echo invalid",
        "runs-on: '${{ success() }}'\nsteps:\n  - run: echo invalid",
    ] {
        assert!(!super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn runner_group_mappings_and_container_options_follow_actions_schema() {
    for yaml in [
        "runs-on: {group: ubuntu-runners}\nsteps:\n  - run: echo valid",
        "runs-on: {group: ubuntu-runners, labels: ubuntu-latest}\nsteps:\n  - run: echo valid",
        "runs-on: {labels: [self-hosted, linux]}\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, options: '--cpus 1'}\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\nservices: {postgres: {image: postgres:16, options: '--entrypoint postgres'}}\nsteps:\n  - run: echo valid",
    ] {
        assert!(super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "runs-on: {}\nsteps:\n  - run: echo invalid",
        "runs-on: {pool: ubuntu-runners}\nsteps:\n  - run: echo invalid",
        "runs-on: {group: 1}\nsteps:\n  - run: echo invalid",
        "runs-on: {labels: []}\nsteps:\n  - run: echo invalid",
        "runs-on: {labels: [ubuntu-latest, 1]}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, options: '--entrypoint=/bin/false'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nservices: {postgres: {image: postgres:16, options: '--network=host'}}\nsteps:\n  - run: echo invalid",
    ] {
        assert!(!super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn continue_on_error_uses_its_field_specific_contexts() {
    assert!(super::step_job_shape_valid(&job(
        "continue-on-error: '${{ matrix.experimental }}'\nruns-on: ubuntu-latest\nsteps:\n  - run: echo valid"
    )));
    assert!(!super::step_job_shape_valid(&job(
        "continue-on-error: '${{ steps.setup.outputs.allowed }}'\nruns-on: ubuntu-latest\nsteps:\n  - run: echo invalid"
    )));
    assert!(!super::step_job_shape_valid(&job(
        "continue-on-error: '${{ failure() }}'\nruns-on: ubuntu-latest\nsteps:\n  - run: echo invalid"
    )));
    for yaml in [
        "steps:\n  - continue-on-error: '${{ steps.setup.outputs.allowed }}'\n    run: echo valid",
        "steps:\n  - continue-on-error: '${{ secrets.ALLOW_FAILURES || hashFiles(''**/pnpm-lock.yaml'') != '''' }}'\n    run: echo valid",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }
    let invalid_step_status =
        "steps:\n  - continue-on-error: '${{ failure() }}'\n    run: echo invalid";
    assert!(
        !steps_shape_valid(&job(invalid_step_status)),
        "{invalid_step_status}"
    );
}

#[test]
fn continue_on_error_rejects_static_nonboolean_expressions() {
    for yaml in [
        "continue-on-error: \"${{ 'false' }}\"\nruns-on: ubuntu-latest\nsteps:\n  - run: echo invalid",
        "continue-on-error: '${{ 1 }}'\nruns-on: ubuntu-latest\nsteps:\n  - run: echo invalid",
        "continue-on-error: '${{ null }}'\nruns-on: ubuntu-latest\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: \"${{ 'false' }}\"\n    run: echo invalid",
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: '${{ 1 }}'\n    run: echo invalid",
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: '${{ null }}'\n    run: echo invalid",
    ] {
        assert!(!if yaml.starts_with("runs-on:") {
            steps_shape_valid(&job(yaml))
        } else {
            super::step_job_shape_valid(&job(yaml))
        }, "{yaml}");
    }
    for yaml in [
        "continue-on-error: '${{ inputs.allowed }}'\nruns-on: ubuntu-latest\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\nsteps:\n  - continue-on-error: '${{ steps.setup.outputs.allowed }}'\n    run: echo valid",
    ] {
        assert!(if yaml.starts_with("runs-on:") {
            steps_shape_valid(&job(yaml))
        } else {
            super::step_job_shape_valid(&job(yaml))
        }, "{yaml}");
    }
}

#[test]
fn timeout_minutes_uses_its_field_specific_contexts_and_functions() {
    for yaml in [
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ matrix.timeout }}'\nstrategy:\n  matrix:\n    timeout: [5]\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ inputs.timeout }}'\nsteps:\n  - run: echo valid",
        "runs-on: ubuntu-latest\ntimeout-minutes: 361\nsteps:\n  - run: echo valid",
    ] {
        assert!(super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - timeout-minutes: '${{ steps.setup.outputs.timeout }}'\n    run: echo valid",
        "steps:\n  - timeout-minutes: '${{ github.run_number }}'\n    run: echo valid",
        "steps:\n  - timeout-minutes: \"${{ case(contains(hashFiles('**/pnpm-lock.yaml'), 'x'), 5, 10) }}\"\n    run: echo valid",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }

    for yaml in [
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ secrets.TIMEOUT }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ hashFiles(''**/pnpm-lock.yaml'') }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ failure() }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ false }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ntimeout-minutes: '${{ 0 }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ntimeout-minutes: \"${{ '5' }}\"\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ntimeout-minutes: \"${{ fromJSON('not-json') }}\"\nsteps:\n  - run: echo invalid",
    ] {
        assert!(!super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - timeout-minutes: '${{ failure() }}'\n    run: echo invalid",
        "steps:\n  - timeout-minutes: '${{ -1 }}'\n    run: echo invalid",
        "steps:\n  - timeout-minutes: '${{ 361 }}'\n    run: echo invalid",
        "steps:\n  - timeout-minutes: \"${{ fromJSON('not-json') }}\"\n    run: echo invalid",
        "steps:\n  - timeout-minutes: '${{ 361 || 5 }}'\n    run: echo invalid",
    ] {
        assert!(!steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn environments_use_distinct_name_and_url_contexts() {
    assert!(super::step_job_shape_valid(&job(
        "runs-on: ubuntu-latest\nenvironment:\n  name: '${{ matrix.deployment }}'\n  url: '${{ steps.deploy.outputs.url }}'\nsteps:\n  - run: echo valid"
    )));
    for yaml in [
        "runs-on: ubuntu-latest\nenvironment: '${{ secrets.DEPLOY_ENV }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nenvironment: {name: '${{ steps.deploy.outputs.name }}'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nenvironment: {name: staging, url: '${{ secrets.DEPLOY_URL }}'}\nsteps:\n  - run: echo invalid",
    ] {
        assert!(!super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn environment_url_rechecks_resolved_stringable_values_per_active_state() {
    let input_url = job(
        "runs-on: ubuntu-latest\nenvironment:\n  name: staging\n  url: '${{ inputs.url }}'\nsteps:\n  - run: echo valid",
    );
    let matrix_url = job(
        "runs-on: ubuntu-latest\nenvironment:\n  name: staging\n  url: '${{ matrix.url }}'\nsteps:\n  - run: echo valid",
    );
    let mut inputs = InputState::new();

    inputs.insert(
        "url".to_string(),
        StaticValue::String("https://example.test".to_string()),
    );
    assert!(super::environment_configuration_valid_for_inputs(
        &input_url, &inputs
    ));

    inputs.insert("url".to_string(), StaticValue::Bool(true));
    assert!(super::environment_configuration_valid_for_inputs(
        &input_url, &inputs
    ));

    inputs.insert("url".to_string(), StaticValue::Mapping);
    assert!(!super::environment_configuration_valid_for_inputs(
        &input_url, &inputs
    ));

    inputs.insert("url".to_string(), StaticValue::Unknown);
    assert!(super::environment_configuration_valid_for_inputs(
        &input_url, &inputs
    ));

    let matrix_inputs = inputs_with_matrix_values(
        &InputState::new(),
        &BTreeMap::from([(
            String::from("url"),
            Value::Mapping(serde_yaml::Mapping::new()),
        )]),
        MatrixState::Static,
    );
    assert!(!super::environment_configuration_valid_for_inputs(
        &matrix_url,
        &matrix_inputs
    ));
}

#[test]
fn runs_on_uses_only_runner_selection_contexts() {
    for yaml in [
        "runs-on: '${{ github.ref_name }}'\nsteps:\n  - run: echo valid",
        "runs-on: ['self-hosted', '${{ matrix.runner }}']\nstrategy:\n  matrix: {runner: [linux]}\nsteps:\n  - run: echo valid",
        "runs-on: '${{ needs.prepare.outputs.runner }}'\nsteps:\n  - run: echo valid",
        "runs-on: '${{ vars.RUNNER }}'\nsteps:\n  - run: echo valid",
    ] {
        assert!(super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "runs-on: '${{ secrets.RUNNER }}'\nsteps:\n  - run: echo invalid",
        "runs-on: '${{ jobs.build.outputs.runner }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ['self-hosted', '${{ runner.os }}']\nsteps:\n  - run: echo invalid",
        "runs-on: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\nsteps:\n  - run: echo invalid",
    ] {
        assert!(!super::step_job_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn action_steps_require_static_canonical_targets() {
    for yaml in [
        "steps:\n  - uses: actions/checkout@v4",
        "steps:\n  - uses: owner/action/subdirectory@main",
        "steps:\n  - uses: owner/action@release/2026.08",
        "steps:\n  - uses: owner/action@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
        "steps:\n  - uses: ./",
        "steps:\n  - uses: ./.github/actions/check",
        "steps:\n  - uses: docker://alpine:3.8",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - uses: actions/checkout",
        "steps:\n  - uses: actions/checkout@${{ github.ref }}",
        "steps:\n  - uses: actions/checkout@bad..ref",
        "steps:\n  - uses: actions/checkout@main branch",
        "steps:\n  - uses: actions/checkout@refs//heads/main",
        "steps:\n  - uses: actions/checkout@release.lock",
        "steps:\n  - uses: owner/../action@v1",
        "steps:\n  - uses: -owner/action@v1",
        "steps:\n  - uses: owner-/action@v1",
        "steps:\n  - uses: ./../outside",
        "steps:\n  - uses: ./.github/actions/${{ matrix.action }}",
        "steps:\n  - uses: docker://",
        "steps:\n  - uses: docker://ghcr.io//checker:22",
        "steps:\n  - uses: docker://ghcr.io/checker:${{ matrix.tag }}",
        "steps:\n  - uses: docker://node:${{ '' }}",
        "steps:\n  - uses: \"docker://${{ 'alpine:3.8' }}\"",
        "steps:\n  - uses: docker://${{ secrets.IMAGE }}",
    ] {
        assert!(!steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn action_step_with_uses_its_field_specific_contexts_and_functions() {
    for yaml in [
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: '${{ github.ref }}'\n      token: '${{ secrets.TOKEN }}'",
        "steps:\n  - id: setup\n    run: echo setup\n  - uses: actions/checkout@v4\n    with:\n      ref: \"${{ format('{0}', steps.setup.outputs.ref) }}\"",
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: '${{ jobs.typecheck.outputs.ref }}'",
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: '${{ success() }}'",
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: \"${{ fromJSON('{}') }}\"",
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: \"${{ fromJSON('[]') }}\"",
        "steps:\n  - uses: actions/checkout@v4\n    with:\n      ref: \"prefix-${{ fromJSON('{}') }}\"",
    ] {
        assert!(!steps_shape_valid(&job(yaml)), "{yaml}");
    }
}

#[test]
fn call_bindings_require_unique_scalar_names() {
    for yaml in [
        "uses: owner/repo/.github/workflows/a.yml@main",
        "with:\n  enabled: true",
        "secrets: inherit",
        "secrets:\n  token: '${{ secrets.TOKEN }}'",
        "secrets:\n  token: '${{ needs.setup.outputs.token }}'",
        "secrets:\n  token: '${{ github.token }}'",
        "secrets:\n  token: \"${{ strategy['job-total'] }}\"",
        "secrets:\n  token: '${{ matrix.token }}'",
        "secrets:\n  token: '${{ inputs.token }}'",
        "secrets:\n  token: '${{ vars.TOKEN }}'",
    ] {
        assert!(call_bindings_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "with: true",
        "with:\n  arg: []",
        "with:\n  Name: yes\n  name: no",
        "secrets: all",
        "secrets: INHERIT",
        "secrets: []",
        "secrets:\n  token: null",
        "secrets:\n  Token: one\n  token: two",
        "secrets:\n  token: '${{ success() }}'",
        "secrets:\n  token: '${{ steps.setup.outputs.token }}'",
        "secrets:\n  token: '${{ env.TOKEN }}'",
    ] {
        assert!(!call_bindings_shape_valid(&job(yaml)), "{yaml}");
    }
}
