//! Preserve SQL AST serialization order without serde_json map feature coupling.
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;
use std::fmt;

pub(super) fn identity(value: &impl serde::Serialize) -> String {
    let serialized = serde_json::to_string(value).expect("serializable SQL expression");
    let mut output = String::with_capacity(serialized.len());
    write(&serialized, &mut output);
    output
}

fn write(raw: &str, output: &mut String) {
    match raw.as_bytes()[0] {
        b'{' => {
            let Fields(mut fields) = serde_json::from_str(raw).expect("serialized SQL object");
            // Legacy preserve_order Map::remove uses swap_remove, not shift_remove.
            if let Some(index) = fields.iter().position(|(key, _)| key == "span") {
                fields.swap_remove(index);
            }
            output.push('{');
            for (index, (key, value)) in fields.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).expect("serializable SQL field"));
                output.push(':');
                write(value.get(), output);
            }
            output.push('}');
        }
        b'[' => {
            let values: Vec<&RawValue> = serde_json::from_str(raw).expect("serialized SQL array");
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                write(value.get(), output);
            }
            output.push(']');
        }
        _ => output.push_str(raw),
    }
}

struct Fields<'a>(Vec<(String, &'a RawValue)>);

impl<'de> Deserialize<'de> for Fields<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldsVisitor;
        impl<'de> Visitor<'de> for FieldsVisitor {
            type Value = Fields<'de>;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("SQL expression object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut fields = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    fields.push(entry);
                }
                Ok(Fields(fields))
            }
        }
        deserializer.deserialize_map(FieldsVisitor)
    }
}

#[cfg(test)]
#[path = "tests/expression_identity.rs"]
mod tests;
