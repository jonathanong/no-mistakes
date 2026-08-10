use super::*;

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn steps_cannot_mix_action_and_shell_commands() {
    assert!(steps_shape_valid(&job("steps:\n  - run: echo ok")));
    assert!(steps_shape_valid(&job("steps:\n  - uses: owner/action@v1")));
    for yaml in [
        "steps: invalid",
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
        "secrets: []",
        "secrets:\n  token: null",
        "secrets:\n  Token: one\n  token: two",
    ] {
        assert!(!call_bindings_shape_valid(&job(yaml)), "{yaml}");
    }
}
