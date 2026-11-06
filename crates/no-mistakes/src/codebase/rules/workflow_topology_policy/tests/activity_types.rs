use crate::codebase::rules::RuleFinding;

fn findings(options: &str) -> Vec<RuleFinding> {
    let root = super::fixture("activity-types");
    super::super::prepare::check_with_files(&root, &super::config(options), &[])
        .unwrap()
        .into_iter()
        .filter(|finding| !finding.file.contains("suppressed-"))
        .collect()
}

#[test]
fn normalizes_scalar_and_sequence_types_across_events_and_locates_types_keys() {
    let found =
        findings("forbiddenActivityTypes: [{types: [ready_for_review, converted_to_draft]}]");
    assert_eq!(found.len(), 3);
    assert!(found
        .iter()
        .all(|finding| finding.file.ends_with("events.yml")));
    assert_eq!(
        found.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        vec![3, 5, 7]
    );
    assert!(found[0].message.contains("pull_request: ready_for_review"));
    assert!(found[1]
        .message
        .contains("pull_request_target: converted_to_draft"));
    assert!(found[2].message.contains("issues: ready_for_review"));
    assert!(found
        .iter()
        .all(|finding| finding.message.contains("remove it from on.")));
}

#[test]
fn restricts_events_and_distinguishes_explicit_empty_filter() {
    let found = findings("forbiddenActivityTypes: [{types: [ready_for_review], events: [issues]}]");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 7);
    assert!(
        findings("forbiddenActivityTypes: [{types: [ready_for_review], events: []}]").is_empty()
    );
    assert!(findings("forbiddenActivityTypes: [{types: [], events: [issues]}]").is_empty());
    assert!(findings("{}").is_empty());
}

#[test]
fn merges_overlapping_policies_without_duplicate_findings() {
    let found = findings(
        "forbiddenActivityTypes:\n  - types: [ready_for_review]\n  - types: [ready_for_review]\n    events: [pull_request]\n",
    );
    assert_eq!(found.len(), 2);
    assert_eq!(
        found.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        vec![3, 7]
    );
}

#[test]
fn requires_typed_configuration_and_rejects_unknown_policy_fields() {
    for yaml in [
        "forbiddenActivityTypes: [{events: [issues]}]",
        "forbiddenActivityTypes: [{types: ready_for_review}]",
        "forbiddenActivityTypes: [{types: [ready_for_review], events: issues}]",
        "forbiddenActivityTypes: [{types: [ready_for_review], event: issues}]",
    ] {
        assert!(
            super::config(yaml).rules[0]
                .try_rule_options::<super::super::Options>()
                .is_err(),
            "{yaml}"
        );
    }
}

#[test]
fn custom_messages_apply_only_to_their_rule_application() {
    let root = super::fixture("activity-types");
    let mut config =
        super::config("forbiddenActivityTypes: [{types: [ready_for_review], events: [issues]}]");
    config.rules[0].message = Some("Remove this unwanted activity type.".to_owned());
    config
        .rules
        .extend(super::config("forbiddenActivityTypes: [{types: [converted_to_draft]}]").rules);
    let found = super::super::prepare::check_with_files(&root, &config, &[]).unwrap();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].line, 5);
    assert!(found[0].message.contains("converted_to_draft"));
    assert_eq!(found[1].line, 7);
    assert_eq!(found[1].message, "Remove this unwanted activity type.");
}
