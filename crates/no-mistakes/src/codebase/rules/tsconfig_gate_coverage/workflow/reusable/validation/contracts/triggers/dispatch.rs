use serde_yaml::{Mapping, Value};

const WORKFLOW_DISPATCH_INPUT_LIMIT: usize = 10;

pub(super) fn workflow_dispatch_config_valid(config: &Value) -> bool {
    config.as_mapping().is_some_and(|mapping| {
        only_keys(mapping, &["inputs"])
            && mapping
                .get("inputs")
                .is_none_or(workflow_dispatch_inputs_valid)
    })
}

fn workflow_dispatch_inputs_valid(inputs: &Value) -> bool {
    inputs.as_mapping().is_some_and(|inputs| {
        inputs.len() <= WORKFLOW_DISPATCH_INPUT_LIMIT
            && inputs.iter().all(|(name, declaration)| {
                name.is_string()
                    && declaration
                        .as_mapping()
                        .is_some_and(workflow_dispatch_input_valid)
            })
    })
}

fn workflow_dispatch_input_valid(declaration: &Mapping) -> bool {
    only_keys(
        declaration,
        &["description", "required", "default", "type", "options"],
    ) && string_field_valid(declaration, "description")
        && bool_field_valid(declaration, "required")
        && declaration.get("type").is_none_or(|input_type| {
            input_type.as_str().is_some_and(|input_type| {
                matches!(
                    input_type,
                    "boolean" | "choice" | "environment" | "number" | "string"
                )
            })
        })
        && declaration
            .get("options")
            .is_none_or(non_empty_string_sequence)
        && workflow_dispatch_input_values_valid(declaration)
}

fn workflow_dispatch_input_values_valid(declaration: &Mapping) -> bool {
    let input_type = declaration.get("type").and_then(Value::as_str);
    let default = declaration.get("default");
    match input_type {
        Some("boolean") => {
            declaration.get("options").is_none() && default.is_none_or(Value::is_bool)
        }
        Some("number") => {
            declaration.get("options").is_none() && default.is_none_or(Value::is_number)
        }
        Some("choice") => declaration.get("options").is_some_and(|options| {
            non_empty_string_sequence(options)
                && default.is_none_or(|default| {
                    default.as_str().is_some_and(|default| {
                        options.as_sequence().is_some_and(|options| {
                            options
                                .iter()
                                .any(|option| option.as_str() == Some(default))
                        })
                    })
                })
        }),
        Some("environment" | "string") | None => {
            declaration.get("options").is_none() && default.is_none_or(Value::is_string)
        }
        Some(_) => false,
    }
}

fn non_empty_string_sequence(value: &Value) -> bool {
    value.as_sequence().is_some_and(|values| {
        !values.is_empty()
            && values
                .iter()
                .all(|value| value.as_str().is_some_and(|value| !value.is_empty()))
    })
}

fn only_keys(mapping: &Mapping, allowed: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| allowed.contains(&key)))
}

fn string_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(Value::is_string)
}

fn bool_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(Value::is_bool)
}
