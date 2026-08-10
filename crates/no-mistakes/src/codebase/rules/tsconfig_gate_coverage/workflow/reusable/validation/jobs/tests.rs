use super::*;
use serde_yaml::Value;

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
        "steps:\n  - run: 'echo ${{ }}'",
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
fn action_steps_require_static_canonical_targets() {
    for yaml in [
        "steps:\n  - uses: actions/checkout@v4",
        "steps:\n  - uses: owner/action/subdirectory@main",
        "steps:\n  - uses: ./.github/actions/check",
        "steps:\n  - uses: docker://alpine:3.8",
    ] {
        assert!(steps_shape_valid(&job(yaml)), "{yaml}");
    }
    for yaml in [
        "steps:\n  - uses: actions/checkout",
        "steps:\n  - uses: actions/checkout@${{ github.ref }}",
        "steps:\n  - uses: ./../outside",
        "steps:\n  - uses: docker://",
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
    ] {
        assert!(!call_bindings_shape_valid(&job(yaml)), "{yaml}");
    }
}
