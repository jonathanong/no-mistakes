use super::non_empty_literal_string_sequence;
use serde_yaml::Value;

pub(super) const ACTIVITY_TYPE_TRIGGERS: &[&str] = &[
    "branch_protection_rule",
    "check_run",
    "check_suite",
    "discussion",
    "discussion_comment",
    "issue_comment",
    "issues",
    "label",
    "milestone",
    "pull_request_review",
    "pull_request_review_comment",
    "registry_package",
    "release",
    "watch",
];

pub(super) const PULL_REQUEST_ACTIVITY_TYPES: &[&str] = &[
    "assigned",
    "unassigned",
    "labeled",
    "unlabeled",
    "opened",
    "edited",
    "closed",
    "reopened",
    "synchronize",
    "converted_to_draft",
    "locked",
    "unlocked",
    "enqueued",
    "dequeued",
    "milestoned",
    "demilestoned",
    "ready_for_review",
    "review_requested",
    "review_request_removed",
    "auto_merge_enabled",
    "auto_merge_disabled",
];

const ACTIVITY_TYPES: &[(&str, &[&str])] = &[
    ("branch_protection_rule", &["created", "edited", "deleted"]),
    (
        "check_run",
        &["created", "rerequested", "completed", "requested_action"],
    ),
    ("check_suite", &["completed", "requested", "rerequested"]),
    (
        "discussion",
        &[
            "created",
            "edited",
            "deleted",
            "transferred",
            "pinned",
            "unpinned",
            "labeled",
            "unlabeled",
            "locked",
            "unlocked",
            "category_changed",
            "answered",
            "unanswered",
        ],
    ),
    ("discussion_comment", &["created", "edited", "deleted"]),
    ("issue_comment", &["created", "edited", "deleted"]),
    (
        "issues",
        &[
            "opened",
            "edited",
            "deleted",
            "transferred",
            "pinned",
            "unpinned",
            "closed",
            "reopened",
            "assigned",
            "unassigned",
            "labeled",
            "unlabeled",
            "locked",
            "unlocked",
            "milestoned",
            "demilestoned",
            "typed",
            "untyped",
            "field_added",
            "field_removed",
        ],
    ),
    ("label", &["created", "edited", "deleted"]),
    (
        "milestone",
        &["created", "closed", "opened", "edited", "deleted"],
    ),
    ("pull_request_review", &["submitted", "edited", "dismissed"]),
    (
        "pull_request_review_comment",
        &["created", "edited", "deleted"],
    ),
    ("registry_package", &["published", "updated"]),
    (
        "release",
        &[
            "published",
            "unpublished",
            "created",
            "edited",
            "deleted",
            "prereleased",
            "released",
        ],
    ),
    ("watch", &["started"]),
];

pub(super) fn activity_type_config_valid(trigger: &str, config: &Value) -> bool {
    config.as_mapping().is_some_and(|mapping| {
        mapping.len() <= 1
            && mapping.get("types").is_some_and(|types| {
                string_sequence_values_valid(types, activity_types_for(trigger))
            })
            || mapping.is_empty()
    })
}

pub(super) fn string_sequence_values_valid(value: &Value, allowed: &[&str]) -> bool {
    non_empty_literal_string_sequence(value)
        && value.as_sequence().is_some_and(|values| {
            values
                .iter()
                .all(|value| value.as_str().is_some_and(|value| allowed.contains(&value)))
        })
}

fn activity_types_for(trigger: &str) -> &'static [&'static str] {
    ACTIVITY_TYPES
        .iter()
        .find_map(|(name, types)| (*name == trigger).then_some(*types))
        .unwrap_or(&[])
}

#[cfg(test)]
mod tests;
