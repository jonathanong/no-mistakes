use super::*;

fn workflow(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn workflow_shape_requires_known_top_level_keys_and_supported_field_shapes() {
    assert!(workflow_shape_valid(&workflow(
        "name: checks\nrun-name: 'checks-${{ github.ref }}'\non: push\npermissions: read-all\nenv:\n  NODE_ENV: 'test-${{ github.ref }}'\ndefaults:\n  run:\n    shell: bash\n    working-directory: app\nconcurrency:\n  group: 'checks-${{ github.ref }}'\n  cancel-in-progress: true\njobs: {}",
    )));

    for yaml in [
        "on: push\nbogus: true\njobs: {}",
        "on: push\nname: [checks]\njobs: {}",
        "on: push\nrun-name: ''\njobs: {}",
        "on: push\nenv: []\njobs: {}",
        "on: push\nenv: {}\njobs: {}",
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
    ] {
        assert!(!workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
}

#[test]
fn permissions_follow_the_actions_scope_and_access_schema() {
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
        "on: push\npermissions:\n  contents: invalid\njobs: {}",
        "on: push\npermissions:\n  id-token: read\njobs: {}",
        "on: push\npermissions:\n  models: write\njobs: {}",
        "on: push\npermissions:\n  code-quality: write\njobs: {}",
        "on: push\npermissions:\n  vulnerability-alerts: read\njobs: {}",
    ] {
        assert!(!workflow_shape_valid(&workflow(yaml)), "{yaml}");
    }
}
