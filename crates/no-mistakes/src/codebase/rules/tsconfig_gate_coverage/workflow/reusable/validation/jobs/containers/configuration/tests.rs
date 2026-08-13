use super::{container_configuration_valid_for_inputs, EnvironmentState, InputState};
use serde_yaml::Value;

#[test]
fn services_cannot_publish_the_same_static_host_port() {
    for (yaml, expected) in [
        (
            "services: {db: {image: postgres:16, ports: ['5432:5432']}, cache: {image: redis:7, ports: ['5432:6379']}}",
            false,
        ),
        (
            "services: {db: {image: postgres:16, ports: ['5432:5432']}, cache: {image: redis:7, ports: ['6379:6379']}}",
            true,
        ),
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            container_configuration_valid_for_inputs(
                &job,
                &InputState::new(),
                &EnvironmentState::default(),
            ),
            expected,
            "{yaml}"
        );
    }
}
