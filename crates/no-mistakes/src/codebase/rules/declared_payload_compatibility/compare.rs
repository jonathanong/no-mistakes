use super::schema::{equal, Schema};
use serde_json::Value;

pub(super) fn subset(producer: &Schema, consumer: &Schema, path: &str) -> Result<(), String> {
    let kind_compatible = producer.kind == consumer.kind
        || (producer.kind == "integer" && consumer.kind == "number")
        || (producer.kind == "number"
            && consumer.kind == "integer"
            && producer.values.as_ref().is_some_and(|values| {
                values
                    .iter()
                    .all(|value| super::schema::matches_type(value, "integer"))
            }));
    if !kind_compatible {
        return Err(format!(
            "{path}: producer type `{}` is not accepted by `{}`",
            producer.kind, consumer.kind
        ));
    }
    if let Some(accepted) = &consumer.values {
        // Null/boolean types themselves are finite even without an explicit enum.
        let finite = match producer.kind.as_str() {
            "null" => Some(vec![Value::Null]),
            "boolean" => Some(vec![Value::Bool(false), Value::Bool(true)]),
            _ => None,
        };
        let emitted = producer
            .values
            .as_ref()
            .or(finite.as_ref())
            .ok_or_else(|| {
                format!("{path}: unrestricted producer values exceed consumer enum/const")
            })?;
        if emitted
            .iter()
            .any(|v| !accepted.iter().any(|a| equal(v, a)))
        {
            return Err(format!(
                "{path}: producer enum/const includes a value the consumer rejects"
            ));
        }
    }
    if producer.kind == "object" {
        for required in &consumer.required {
            if !producer.required.contains(required) {
                return Err(format!("{path}[{required:?}]: consumer requires a property the producer does not guarantee"));
            }
        }
        for (name, emitted) in &producer.properties {
            if let Some(accepted) = consumer.properties.get(name) {
                subset(emitted, accepted, &format!("{path}[{name:?}]"))?;
            } else if !consumer.additional {
                return Err(format!("{path}[{name:?}]: producer property is forbidden by the closed consumer object"));
            }
        }
        if producer.additional {
            if !consumer.additional {
                return Err(format!("{path}: open producer object permits properties forbidden by closed consumer object"));
            }
            if let Some(name) = consumer
                .properties
                .keys()
                .find(|name| !producer.properties.contains_key(*name))
            {
                return Err(format!("{path}[{name:?}]: open producer permits unconstrained values for a consumer property"));
            }
        }
    } else if let (Some(emitted), Some(accepted)) = (&producer.items, &consumer.items) {
        subset(emitted, accepted, &format!("{path}[]"))?;
    }
    Ok(())
}
