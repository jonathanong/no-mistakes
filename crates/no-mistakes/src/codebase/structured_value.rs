use serde_yaml::Value;
use std::path::Path;

/// Parse YAML or JSONC into a YAML value used by structured config rules.
///
/// JSON and JSONC paths use `jsonc_parser` so `//` comments are not skipped.
/// Other paths use `serde_yaml`. Parse failures return a diagnostic message
/// instead of a silent skip.
pub fn parse_structured_value(path: &Path, source: &str) -> Result<Value, String> {
    if is_jsonc_path(path) {
        parse_jsonc(source)
    } else {
        serde_yaml::from_str(source).map_err(|error| format!("failed to parse YAML ({error})"))
    }
}

fn is_jsonc_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("json") || extension.eq_ignore_ascii_case("jsonc")
        })
}

fn parse_jsonc(source: &str) -> Result<Value, String> {
    let parsed: Option<serde_json::Value> =
        jsonc_parser::parse_to_serde_value(source, &jsonc_parser::ParseOptions::default())
            .map_err(|error| format!("failed to parse JSONC ({error})"))?;
    Ok(json_to_yaml(parsed.unwrap_or(serde_json::Value::Null)))
}

fn json_to_yaml(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(flag) => Value::Bool(flag),
        serde_json::Value::Number(number) => yaml_number(number),
        serde_json::Value::String(text) => Value::String(text),
        serde_json::Value::Array(items) => {
            Value::Sequence(items.into_iter().map(json_to_yaml).collect())
        }
        serde_json::Value::Object(map) => {
            let mut mapping = serde_yaml::Mapping::new();
            for (key, child) in map {
                mapping.insert(Value::String(key), json_to_yaml(child));
            }
            Value::Mapping(mapping)
        }
    }
}

fn yaml_number(number: serde_json::Number) -> Value {
    if let Some(value) = number.as_i64() {
        return Value::Number(value.into());
    }
    if let Some(value) = number.as_u64() {
        return Value::Number(value.into());
    }
    number
        .as_f64()
        .and_then(|value| serde_yaml::from_str(&value.to_string()).ok())
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests;
