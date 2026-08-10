use serde_yaml::Value;

pub(crate) fn has_workflow_call_trigger(on: &Value) -> bool {
    match on {
        Value::String(trigger) => trigger == "workflow_call",
        Value::Sequence(triggers) => triggers
            .iter()
            .any(|trigger| trigger.as_str() == Some("workflow_call")),
        Value::Mapping(triggers) => triggers.get("workflow_call").is_some(),
        _ => false,
    }
}

pub(crate) fn workflow_call_trigger_keys_valid(on: &Value) -> bool {
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
    "image_version",
    "issue_comment",
    "issues",
    "label",
    "merge_group",
    "milestone",
    "page_build",
    "public",
    "pull_request",
    "pull_request_review",
    "pull_request_review_comment",
    "pull_request_target",
    "push",
    "registry_package",
    "release",
    "repository_dispatch",
    "schedule",
    "status",
    "watch",
    "workflow_call",
    "workflow_dispatch",
    "workflow_run",
];
