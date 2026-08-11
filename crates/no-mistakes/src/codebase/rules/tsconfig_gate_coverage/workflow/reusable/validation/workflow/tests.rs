use super::*;

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
        "concurrency:\n  group: checks-${{ vars.ENVIRONMENT }}\n  cancel-in-progress: '${{ inputs.cancel }}'",
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
        "concurrency: checks-${{ github.ref }}",
        "concurrency:\n  group: checks-${{ needs.setup.outputs.key }}\n  cancel-in-progress: '${{ matrix.cancel }}'",
        "concurrency:\n  group: checks-${{ strategy.job-index }}\n  cancel-in-progress: '${{ inputs.cancel }}'",
        "concurrency: checks-${{ vars.ENVIRONMENT }}",
    ] {
        let value = workflow(yaml);
        assert!(
            job_defaults_shape_valid(value.get("defaults"))
                && job_concurrency_shape_valid(value.get("concurrency")),
            "job-level value should be valid: {yaml}"
        );
    }

    for yaml in [
        "defaults:\n  run:\n    shell: '${{ github.ref }}'",
        "defaults:\n  run:\n    working-directory: 'packages/${{ matrix.package }}'",
        "concurrency: checks-${{ env.CI }}",
        "concurrency:\n  group: checks\n  cancel-in-progress: '${{ secrets.CANCEL }}'",
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
fn permissions_follow_the_actions_scope_and_access_schema() {
    assert!(!permission_value_valid(&Value::Number(1.into()), "read"));
    for yaml in [
        "on: push\npermissions: read-all\njobs: {}",
        "on: push\npermissions: write-all\njobs: {}",
        "on: push\npermissions: {}\njobs: {}",
        "on: push\npermissions:\n  contents: read\n  id-token: write\n  models: read\n  repository-projects: write\njobs: {}",
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
        "on: push\npermissions:\n  code-quality: write\njobs: {}",
        "on: push\npermissions:\n  vulnerability-alerts: read\njobs: {}",
    ] {
        assert!(!workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
}
