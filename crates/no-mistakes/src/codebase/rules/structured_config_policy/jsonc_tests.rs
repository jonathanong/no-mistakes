// Output cases moved to cli_rra_snapshot_structured_config_policy.
use super::*;

#[test]
fn forbidden_keys_flag_object_presence() {
    let value: serde_yaml::Value =
        serde_yaml::from_str("entry:\n  object: Function\n  property: bind\n").unwrap();
    let findings = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "entry".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            forbidden_keys: vec!["object".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    assert!(findings[0]
        .message
        .contains("must not contain object key `object`"));
}
