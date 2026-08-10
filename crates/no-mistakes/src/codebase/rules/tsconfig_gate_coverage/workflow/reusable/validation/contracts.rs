use serde_yaml::Value;

pub(crate) fn workflow_call_shape_valid(on: Option<&Value>) -> bool {
    let Some(contract) = on.and_then(|on| on.get("workflow_call")) else {
        return true;
    };
    let Value::Mapping(contract) = contract else {
        return matches!(contract, Value::Null);
    };
    contract
        .keys()
        .all(|key| matches!(key.as_str(), Some("inputs" | "secrets" | "outputs")))
        && declaration_group_valid(contract.get("inputs"), input_declaration_valid)
        && declaration_group_valid(contract.get("secrets"), secret_declaration_valid)
        && declaration_group_valid(contract.get("outputs"), output_declaration_valid)
}

fn declaration_group_valid(
    declarations: Option<&Value>,
    declaration_valid: fn(&serde_yaml::Mapping) -> bool,
) -> bool {
    declarations.is_none_or(|declarations| {
        declarations.as_mapping().is_some_and(|mapping| {
            mapping.iter().all(|(name, declaration)| {
                name.is_string() && declaration.as_mapping().is_some_and(declaration_valid)
            })
        })
    })
}

fn input_declaration_valid(declaration: &serde_yaml::Mapping) -> bool {
    only_keys(declaration, &["type", "required", "default", "description"])
        && declaration
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|input_type| matches!(input_type, "boolean" | "number" | "string"))
        && bool_field_valid(declaration, "required")
        && scalar_field_valid(declaration, "default")
        && string_field_valid(declaration, "description")
}

fn secret_declaration_valid(declaration: &serde_yaml::Mapping) -> bool {
    only_keys(declaration, &["required", "description"])
        && bool_field_valid(declaration, "required")
        && string_field_valid(declaration, "description")
}

fn output_declaration_valid(declaration: &serde_yaml::Mapping) -> bool {
    only_keys(declaration, &["value", "description"])
        && declaration.get("value").is_some_and(Value::is_string)
        && string_field_valid(declaration, "description")
}

fn only_keys(mapping: &serde_yaml::Mapping, allowed: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| allowed.contains(&key)))
}

fn string_field_valid(mapping: &serde_yaml::Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(Value::is_string)
}

fn bool_field_valid(mapping: &serde_yaml::Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(Value::is_bool)
}

fn scalar_field_valid(mapping: &serde_yaml::Mapping, field: &str) -> bool {
    mapping
        .get(field)
        .is_none_or(|value| matches!(value, Value::Bool(_) | Value::Number(_) | Value::String(_)))
}

#[cfg(test)]
mod tests;
