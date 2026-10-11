use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(super) struct Schema {
    pub(super) kind: String,
    pub(super) values: Option<Vec<Value>>,
    pub(super) properties: BTreeMap<String, Schema>,
    pub(super) required: BTreeSet<String>,
    pub(super) additional: bool,
    pub(super) items: Option<Box<Schema>>,
}

pub(super) fn parse(value: &Value, path: &str, depth: usize) -> Result<Schema, String> {
    if depth > 64 {
        return Err(format!("{path}: schema nesting exceeds 64 levels"));
    }
    let object = value
        .as_object()
        .ok_or_else(|| format!("{path}: schema must be an object"))?;
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .filter(|s| {
            matches!(
                *s,
                "object" | "array" | "string" | "number" | "integer" | "boolean" | "null"
            )
        })
        .ok_or_else(|| format!("{path}: requires a supported single `type`"))?;
    validate_keywords(object, kind, path)?;
    let values = scalar_values(object, kind, path)?;
    let mut schema = Schema {
        kind: kind.into(),
        values,
        properties: BTreeMap::new(),
        required: BTreeSet::new(),
        additional: true,
        items: None,
    };
    if kind == "object" {
        parse_object(&mut schema, object, path, depth)?;
    } else if kind == "array" {
        let items = object
            .get("items")
            .ok_or_else(|| format!("{path}: array requires a single `items` schema"))?;
        schema.items = Some(Box::new(parse(items, &format!("{path}[]"), depth + 1)?));
    }
    Ok(schema)
}

fn validate_keywords(
    object: &serde_json::Map<String, Value>,
    kind: &str,
    path: &str,
) -> Result<(), String> {
    for key in object.keys() {
        let allowed = match key.as_str() {
            "type" | "title" | "description" | "$comment" => true,
            "properties" | "required" | "additionalProperties" => kind == "object",
            "items" => kind == "array",
            "enum" | "const" => kind != "object" && kind != "array",
            _ => false,
        };
        if !allowed {
            return Err(format!("{path}: unsupported keyword `{key}` for `{kind}`"));
        }
    }
    for key in ["title", "description", "$comment"] {
        if object.get(key).is_some_and(|value| !value.is_string()) {
            return Err(format!("{path}: annotation `{key}` must be a string"));
        }
    }
    Ok(())
}

fn parse_object(
    schema: &mut Schema,
    object: &serde_json::Map<String, Value>,
    path: &str,
    depth: usize,
) -> Result<(), String> {
    if let Some(properties) = object.get("properties") {
        let properties = properties
            .as_object()
            .ok_or_else(|| format!("{path}: `properties` must be an object"))?;
        for (name, child) in properties {
            schema.properties.insert(
                name.clone(),
                parse(child, &format!("{path}[{name:?}]"), depth + 1)?,
            );
        }
    }
    if let Some(required) = object.get("required") {
        let required = required
            .as_array()
            .ok_or_else(|| format!("{path}: `required` must be a string array"))?;
        for name in required {
            let name = name
                .as_str()
                .ok_or_else(|| format!("{path}: `required` must contain strings"))?;
            if !schema.required.insert(name.into()) {
                return Err(format!("{path}: duplicate required property `{name}`"));
            }
            // Unconstrained required properties need a top schema; outside this subset.
            if !schema.properties.contains_key(name) {
                return Err(format!(
                    "{path}: required property `{name}` has no declared schema"
                ));
            }
        }
    }
    if let Some(additional) = object.get("additionalProperties") {
        schema.additional = additional
            .as_bool()
            .ok_or_else(|| format!("{path}: `additionalProperties` must be boolean"))?;
    }
    Ok(())
}

fn scalar_values(
    object: &serde_json::Map<String, Value>,
    kind: &str,
    path: &str,
) -> Result<Option<Vec<Value>>, String> {
    if object.contains_key("enum") && object.contains_key("const") {
        return Err(format!(
            "{path}: combined `enum` and `const` is unsupported"
        ));
    }
    let values = if let Some(values) = object.get("enum") {
        let values = values
            .as_array()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("{path}: `enum` must be a nonempty array"))?;
        Some(values.clone())
    } else {
        object.get("const").map(|v| vec![v.clone()])
    };
    if let Some(values) = &values {
        for (index, value) in values.iter().enumerate() {
            if value.as_number().is_some_and(|n| {
                n.is_f64()
                    && n.as_f64()
                        .is_some_and(|v| v.abs() >= 9_007_199_254_740_992.0)
            }) {
                return Err(format!(
                    "{path}: floating enum/const value exceeds exact numeric comparison range"
                ));
            }
            if !matches_type(value, kind) {
                return Err(format!("{path}: enum/const value does not match `{kind}`"));
            }
            if values[..index].iter().any(|prior| equal(prior, value)) {
                return Err(format!("{path}: duplicate enum value"));
            }
        }
    }
    Ok(values)
}

pub(super) fn matches_type(value: &Value, kind: &str) -> bool {
    match kind {
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "integer" => value.as_number().is_some_and(|n| {
            n.is_i64() || n.is_u64() || n.as_f64().is_some_and(|v| v.fract() == 0.0)
        }),
        _ => false,
    }
}

pub(super) fn equal(left: &Value, right: &Value) -> bool {
    // Raw numeric tokens are checked for precision loss before parsing. Floats
    // are bounded below 2^53; larger integers retain exact integer equality.
    if let (Some(l), Some(r)) = (left.as_number(), right.as_number()) {
        if l.is_f64() || r.is_f64() {
            return l.as_f64() == r.as_f64();
        }
    }
    left == right
}
