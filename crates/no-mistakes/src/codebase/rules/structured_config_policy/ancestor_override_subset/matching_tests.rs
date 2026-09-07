use super::matching::compile_patterns;
use serde_yaml::{Mapping, Value};

#[test]
fn rejects_invalid_and_non_string_patterns() {
    let mut record = Mapping::new();
    record.insert(
        Value::String("files".to_string()),
        Value::Sequence(vec![Value::Number(1.into())]),
    );
    assert!(compile_patterns(&record, "files", true).is_err());
    record.insert(
        Value::String("files".to_string()),
        Value::String("[".to_string()),
    );
    assert!(compile_patterns(&record, "files", true).is_err());
}
