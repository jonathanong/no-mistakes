use super::*;

#[test]
fn unsupported_network_and_entrypoint_options_are_rejected() {
    for value in [
        "--network host",
        "--network=host",
        "'--network' host",
        "\"--network=host\"",
    ] {
        assert!(!supported(value, ContainerKind::Job), "{value}");
        assert!(!supported(value, ContainerKind::Service), "{value}");
    }
    for value in ["--entrypoint /bin/false", "--entrypoint=/bin/false"] {
        assert!(!supported(value, ContainerKind::Job), "{value}");
        assert!(supported(value, ContainerKind::Service), "{value}");
    }
    for value in ["--cpus 1", "--env 'NAME=value with spaces'"] {
        assert!(supported(value, ContainerKind::Job), "{value}");
    }
    for value in [
        "--networking enabled",
        "--env NAME=value\\ with\\ spaces",
        "--env \"NAME='value'\"",
    ] {
        assert!(supported(value, ContainerKind::Job), "{value}");
    }
    for value in ["", "'unterminated", "dangling\\"] {
        assert!(!supported(value, ContainerKind::Job), "{value}");
    }
}

#[test]
fn dynamic_options_remain_conservative_until_inputs_resolve() {
    assert!(shape_valid("${{ github.ref }}", ContainerKind::Job));
    assert!(!shape_valid("${{ }}", ContainerKind::Job));
    assert!(valid_for_inputs(
        Some("${{ github.ref }}"),
        ContainerKind::Job,
        &InputState::new(),
        &EnvironmentState::default(),
    ));
    assert!(valid_for_inputs(
        None,
        ContainerKind::Service,
        &InputState::new(),
        &EnvironmentState::default(),
    ));
}
