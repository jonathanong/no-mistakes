// This executes inside the async native task, including for file and JSON-text inputs.
fn decamelize_audit_artifact(value: serde_json::Value) -> AnyhowResult<serde_json::Value> {
    match value {
        serde_json::Value::Array(values) => values
            .into_iter()
            .map(decamelize_audit_artifact)
            .collect::<AnyhowResult<Vec<_>>>()
            .map(serde_json::Value::Array),
        serde_json::Value::Object(values) => {
            let mut normalized = serde_json::Map::new();
            for (key, value) in values {
                let mut name = String::new();
                for character in key.chars() {
                    if character.is_ascii_uppercase() {
                        name.push('_');
                        name.push(character.to_ascii_lowercase());
                    } else {
                        name.push(character);
                    }
                }
                if normalized
                    .insert(name.clone(), decamelize_audit_artifact(value)?)
                    .is_some()
                {
                    bail!("Ambiguous audit artifact field: {name}");
                }
            }
            Ok(serde_json::Value::Object(normalized))
        }
        scalar => Ok(scalar),
    }
}
