//! Lenient value coercion helpers, ported from `workflow-value-primitives.mts`.
//!
//! GitHub Actions YAML is parsed leniently: a `serde_yaml::Value` walked by
//! hand rather than deserialized into a strict struct, so a malformed or
//! oddly-typed field degrades gracefully (silently ignored / coerced)
//! instead of failing the whole parse — matching the original engine's
//! `unknown`-typed, hand-walked TS parsing.

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_yaml::Value;

/// Accepts a single string OR a sequence of strings; non-string sequence
/// items are silently dropped. Anything else yields an empty list.
pub fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) => vec![text.clone()],
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

/// Coerces a string, number, or boolean scalar to a string (matching JS
/// `String(value)`); anything else (including absent) yields `None`.
pub fn string_value(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

/// True for a YAML mapping (the analog of a non-array JS object).
pub fn is_record(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::Mapping(_)))
}

/// `concurrency.cancel-in-progress` accepts a literal boolean, or anything
/// [`string_value`] can coerce.
pub fn concurrency_value(value: Option<&Value>) -> Option<super::model::ConcurrencyValue> {
    match value {
        Some(Value::Bool(flag)) => Some(super::model::ConcurrencyValue::Bool(*flag)),
        other => string_value(other).map(super::model::ConcurrencyValue::Text),
    }
}

/// A `serde_json::Value` analog that preserves YAML mapping key order on
/// serialization regardless of whether the `serde_json` crate's
/// `preserve_order` feature is enabled crate-wide (it isn't, here).
///
/// [`to_json`] (trigger `config`, job `matrix`) is the one place this
/// module needs to snapshot arbitrary YAML **verbatim**, in source-document
/// order — matching the TS engine's `toJson`, which round-trips through
/// plain JS objects that preserve string-key insertion order. Every other
/// JSON-object-shaped field in this module (`workflowCall.inputs`, etc.) is
/// instead an explicitly re-sorted typed contract using `BTreeMap`, which
/// needs no special handling. `serde_json::Map` defaults to a `BTreeMap`
/// (alphabetically sorted) without that feature, which would silently
/// reorder a raw snapshot's keys — this type exists so that can't happen.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderedJson {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<OrderedJson>),
    Object(Vec<(String, OrderedJson)>),
}

impl OrderedJson {
    /// Linear-scan lookup by key — snapshot objects here are small
    /// (a handful of `on:`/`strategy.matrix` fields), so this is cheap and
    /// avoids needing a second, hashed representation just for lookups.
    pub fn get(&self, key: &str) -> Option<&OrderedJson> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[OrderedJson]> {
        match self {
            Self::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }
}

impl Serialize for OrderedJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(flag) => serializer.serialize_bool(*flag),
            Self::Number(number) => number.serialize(serializer),
            Self::String(text) => serializer.serialize_str(text),
            Self::Array(items) => items.serialize(serializer),
            Self::Object(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

/// Converts a `serde_yaml::Value` to an [`OrderedJson`] snapshot.
/// Non-scalar/array/map YAML values (tags, timestamps, etc.) fall back to
/// a string representation, mirroring the TS engine's `String(value)`
/// fallback — GitHub Actions workflow YAML essentially never reaches that
/// branch.
pub fn to_json(value: &Value) -> OrderedJson {
    match value {
        Value::Null => OrderedJson::Null,
        Value::Bool(flag) => OrderedJson::Bool(*flag),
        Value::Number(number) => yaml_number_to_json(number)
            .map(OrderedJson::Number)
            .unwrap_or(OrderedJson::Null),
        Value::String(text) => OrderedJson::String(text.clone()),
        Value::Sequence(items) => OrderedJson::Array(items.iter().map(to_json).collect()),
        Value::Mapping(mapping) => OrderedJson::Object(
            mapping
                .iter()
                .map(|(key, item)| {
                    let key = key.as_str().map(str::to_string).unwrap_or_else(|| {
                        string_value(Some(key)).unwrap_or_else(|| format!("{key:?}"))
                    });
                    (key, to_json(item))
                })
                .collect(),
        ),
        Value::Tagged(tagged) => to_json(&tagged.value),
    }
}

/// Converts a YAML number to its JSON equivalent. Integers round-trip
/// exactly; a float that can't be represented as a finite JSON number
/// (`NaN`/`Infinity`, which YAML technically allows) yields `None` — an
/// unreachable case for realistic GitHub Actions workflow YAML.
pub fn yaml_number_to_json(number: &serde_yaml::Number) -> Option<serde_json::Number> {
    number
        .as_i64()
        .map(serde_json::Number::from)
        .or_else(|| number.as_u64().map(serde_json::Number::from))
        .or_else(|| number.as_f64().and_then(serde_json::Number::from_f64))
}
