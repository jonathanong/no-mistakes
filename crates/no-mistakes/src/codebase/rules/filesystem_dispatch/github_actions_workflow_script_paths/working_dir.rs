use super::scripts::{self, WorkingDir};
use serde_yaml::Value;

pub(super) fn parse_working_dir(value: &Value) -> WorkingDir {
    match value {
        Value::Null => WorkingDir::Known(String::new()),
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.contains("${{") || trimmed.contains('$') {
                WorkingDir::Dynamic(trimmed.to_string())
            } else {
                scripts::resolve_rel_path("", trimmed)
                    .map_or(WorkingDir::OutsideRepo, WorkingDir::Known)
            }
        }
        _ => WorkingDir::Dynamic("non-string".to_string()),
    }
}
