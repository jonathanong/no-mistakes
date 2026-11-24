mod merge_keys;
mod merge_locations;

pub(crate) use merge_keys::expand_merge_keys;
use merge_locations::MergeLocations;
use serde::Deserialize;
use serde_yaml::Value;
use std::borrow::Cow;
use std::path::Path;

fn expand_with_context(value: &mut Value) -> Result<(), String> {
    expand_merge_keys(value).map_err(|error| merge_error(&error))
}

pub(crate) fn expand_yaml_mapping_if_merged<'a>(
    value: &'a Value,
    source: Option<&str>,
) -> Result<Cow<'a, Value>, String> {
    if !contains_merge(value) {
        return Ok(Cow::Borrowed(value));
    }
    let mut expanded = value.clone();
    if let Some(source) = source {
        MergeLocations::parse(source)
            .expand(0, &mut expanded)
            .map_err(|(_, error)| merge_error(&error))?;
    } else {
        expand_with_context(&mut expanded)?;
    }
    Ok(Cow::Owned(expanded))
}

fn contains_merge(value: &Value) -> bool {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Mapping(mapping) => {
                if mapping.contains_key("<<") {
                    return true;
                }
                pending.extend(mapping.values());
            }
            Value::Sequence(items) => pending.extend(items),
            Value::Tagged(tagged) => pending.push(&tagged.value),
            _ => {}
        }
    }
    false
}

fn merge_error(error: &str) -> String {
    format!("failed to apply YAML merge keys ({error})")
}

pub(crate) struct YamlDocuments {
    pub(crate) values: Vec<Value>,
    pub(crate) errors: Vec<(usize, String)>,
}

/// Parse YAML or JSONC into a YAML value used by structured config rules.
///
/// JSON and JSONC paths use `jsonc_parser` so `//` comments are not skipped.
/// Other paths use `serde_yaml`, then expand YAML 1.1 merge keys (`<<`).
/// Parse failures return a diagnostic message instead of a silent skip.
pub fn parse_structured_value(path: &Path, source: &str) -> Result<Value, String> {
    parse_structured_value_located(path, source).map_err(|(line, message)| match line {
        Some(line) => format!("{message} at line {line}"),
        None => message,
    })
}

pub(crate) fn parse_structured_value_located(
    path: &Path,
    source: &str,
) -> Result<Value, (Option<usize>, String)> {
    if is_jsonc_path(path) {
        return parse_jsonc(source).map_err(|message| (None, message));
    }
    let mut value: Value = serde_yaml::from_str(source)
        .map_err(|error| (None, format!("failed to parse YAML ({error})")))?;
    if contains_merge(&value) {
        MergeLocations::parse(source)
            .expand(0, &mut value)
            .map_err(|(line, error)| (line, merge_error(&error)))?;
    }
    Ok(value)
}

/// Every YAML document in `source`. Documents parsed before a syntax error
/// are kept. `parse_structured_value` still rejects multiple documents.
pub(crate) fn parse_yaml_documents(source: &str) -> YamlDocuments {
    let mut values = Vec::new();
    let mut errors = Vec::new();
    let mut locations = None;
    for (doc_index, document) in serde_yaml::Deserializer::from_str(source).enumerate() {
        match Value::deserialize(document) {
            Ok(mut value) => {
                if contains_merge(&value) {
                    let locations = locations.get_or_insert_with(|| MergeLocations::parse(source));
                    if let Err((line, error)) = locations.expand(doc_index, &mut value) {
                        errors.push((
                            line.unwrap_or_else(|| locations.document_start(doc_index)),
                            merge_error(&error),
                        ));
                        continue;
                    }
                }
                values.push(value);
            }
            Err(error) => {
                let reported = error
                    .location()
                    .map(|location| location.line())
                    .unwrap_or(1);
                let last_line = source.lines().count().max(1);
                let line = reported.min(last_line);
                let message = format!("failed to parse YAML ({error})");
                // A syntax error can make the YAML parser repeat one document.
                let repeated = errors.last().is_some_and(|previous: &(usize, String)| {
                    previous.0 == line && previous.1 == message
                });
                if repeated {
                    break;
                }
                errors.push((line, message));
            }
        }
    }
    YamlDocuments { values, errors }
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

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
