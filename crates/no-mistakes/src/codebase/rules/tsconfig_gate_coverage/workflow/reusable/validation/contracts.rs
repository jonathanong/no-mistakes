use serde_yaml::Value;

pub(crate) fn workflow_call_shape_valid(on: Option<&Value>) -> bool {
    let Some(on) = on else {
        return true;
    };
    if !workflow_call_trigger_keys_valid(on) {
        return false;
    }
    if !has_workflow_call_trigger(on) {
        return true;
    }
    let Some(contract) = on.get("workflow_call") else {
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

fn has_workflow_call_trigger(on: &Value) -> bool {
    match on {
        Value::String(trigger) => trigger == "workflow_call",
        Value::Sequence(triggers) => triggers
            .iter()
            .any(|trigger| trigger.as_str() == Some("workflow_call")),
        Value::Mapping(triggers) => triggers.get("workflow_call").is_some(),
        _ => false,
    }
}

fn workflow_call_trigger_keys_valid(on: &Value) -> bool {
    match on {
        Value::String(trigger) => KNOWN_WORKFLOW_TRIGGERS.contains(&trigger.as_str()),
        Value::Sequence(triggers) => triggers.iter().all(|trigger| {
            trigger
                .as_str()
                .is_some_and(|trigger| KNOWN_WORKFLOW_TRIGGERS.contains(&trigger))
        }),
        Value::Mapping(triggers) => triggers.keys().all(|key| {
            key.as_str()
                .is_some_and(|trigger| KNOWN_WORKFLOW_TRIGGERS.contains(&trigger))
        }),
        _ => false,
    }
}

const KNOWN_WORKFLOW_TRIGGERS: &[&str] = &[
    "branch_protection_rule",
    "check_run",
    "check_suite",
    "create",
    "delete",
    "deployment",
    "deployment_status",
    "discussion",
    "discussion_comment",
    "fork",
    "gollum",
    "issue_comment",
    "issues",
    "label",
    "merge_group",
    "milestone",
    "page_build",
    "project",
    "project_card",
    "project_column",
    "public",
    "pull_request",
    "pull_request_review",
    "pull_request_review_comment",
    "pull_request_target",
    "push",
    "registry_package",
    "release",
    "repository",
    "repository_dispatch",
    "repository_import",
    "repository_vulnerability_alert",
    "schedule",
    "secret_scanning_alert",
    "status",
    "team_add",
    "watch",
    "workflow_call",
    "workflow_dispatch",
    "workflow_job",
    "workflow_run",
];

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
