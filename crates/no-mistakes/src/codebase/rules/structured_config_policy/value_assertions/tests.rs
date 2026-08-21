use super::super::{AssertionKind, ValueAssertion};
use super::kinds::kind_violation;
use serde_yaml::Value;

#[test]
fn equals_file_is_not_evaluated_per_value() {
    let assertion = ValueAssertion {
        kind: Some(AssertionKind::EqualsFile),
        ..Default::default()
    };
    assert!(kind_violation(&Value::Null, &assertion, AssertionKind::EqualsFile).is_none());
}
