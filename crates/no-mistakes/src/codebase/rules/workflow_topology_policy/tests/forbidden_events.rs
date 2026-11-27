use no_mistakes_rule_kernel::finding::RuleFinding;

fn findings(options: &str) -> Vec<RuleFinding> {
    let root = super::fixture("forbidden-events");
    super::super::prepare::check_with_files(&root, &super::config(options), &[])
        .unwrap()
        .into_iter()
        .filter(|finding| !finding.file.contains("suppressed-"))
        .collect()
}

fn lines_in(found: &[RuleFinding], file: &str) -> Vec<usize> {
    found
        .iter()
        .filter(|finding| finding.file.ends_with(&format!("/{file}")))
        .map(|finding| finding.line)
        .collect()
}

#[test]
fn reports_scalar_sequence_and_mapping_forms_on_the_offending_key() {
    let found = findings("forbiddenEvents: [pull_request_target]");
    assert_eq!(lines_in(&found, "scalar.yml"), vec![2]);
    assert_eq!(lines_in(&found, "sequence.yml"), vec![2]);
    assert_eq!(lines_in(&found, "sequence-block.yml"), vec![4]);
    assert_eq!(lines_in(&found, "duplicate.yml"), vec![2]);
    assert_eq!(lines_in(&found, "mapping.yml"), vec![3]);
    assert_eq!(lines_in(&found, "null-mapping.yml"), vec![3]);
    assert!(lines_in(&found, "case.yml").is_empty());
    assert!(lines_in(&found, "scalar-allowed.yml").is_empty());
    assert!(lines_in(&found, "sequence-allowed.yml").is_empty());
    assert!(lines_in(&found, "mapping-allowed.yml").is_empty());
    assert!(found.iter().all(|finding| finding.message.contains(
        "forbidden workflow event: pull_request_target; remove it from on to avoid unwanted workflow runs"
    )));
    assert_eq!(found.len(), 14);
}

#[test]
fn reports_quoted_flow_comment_and_unmapped_forms_on_the_event_line() {
    let found = findings("forbiddenEvents: [pull_request_target]");
    assert_eq!(lines_in(&found, "quoted-event.yml"), vec![3]);
    assert_eq!(lines_in(&found, "quoted-on.yml"), vec![3]);
    assert_eq!(lines_in(&found, "flow-mapping.yml"), vec![2]);
    assert_eq!(lines_in(&found, "flow-sequence.yml"), vec![4]);
    assert_eq!(lines_in(&found, "comment-gap.yml"), vec![4]);
    assert_eq!(lines_in(&found, "stale-null.yml"), vec![3]);
    assert_eq!(lines_in(&found, "stale-sequence.yml"), vec![3]);
    assert_eq!(lines_in(&found, "tagged-on.yml"), vec![1]);
    assert!(lines_in(&found, "multi-document.yml").is_empty());
}

#[test]
fn reports_each_distinct_forbidden_event_once() {
    let found = findings("forbiddenEvents: [pull_request_target, workflow_dispatch]");
    assert_eq!(lines_in(&found, "mapping.yml"), vec![3, 5]);
    let mapping: Vec<_> = found
        .iter()
        .filter(|finding| finding.file.ends_with("/mapping.yml"))
        .map(|finding| finding.message.as_str())
        .collect();
    assert!(mapping[0].contains("pull_request_target"));
    assert!(mapping[1].contains("workflow_dispatch"));
    assert_eq!(lines_in(&found, "duplicate.yml"), vec![2]);
}

#[test]
fn omitted_and_empty_lists_ban_nothing() {
    assert!(findings("{}").is_empty());
    assert!(findings("forbiddenEvents: []").is_empty());
}

#[test]
fn rejects_a_non_list_event_ban() {
    for yaml in ["forbiddenEvents: pull_request_target", "forbiddenEvents: 1"] {
        assert!(
            super::config(yaml).rules[0]
                .try_rule_options::<super::super::Options>()
                .is_err(),
            "{yaml}"
        );
    }
}

#[test]
fn custom_messages_replace_only_that_application() {
    let root = super::fixture("forbidden-events");
    let mut config = super::config("forbiddenEvents: [pull_request_target]");
    config.rules[0].message = Some("Use pull_request for these widget workflows.".to_owned());
    let found = super::super::prepare::check_with_files(&root, &config, &[])
        .unwrap()
        .into_iter()
        .filter(|finding| !finding.file.contains("suppressed-"))
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 14);
    assert!(found
        .iter()
        .all(|finding| finding.message == "Use pull_request for these widget workflows."));
}

#[test]
fn directive_lines_are_the_event_key_for_mapping_and_sequence_forms() {
    // check_with_files defers suppression. The CLI snapshot drops these.
    let root = super::fixture("forbidden-events");
    let found = super::super::prepare::check_with_files(
        &root,
        &super::config("forbiddenEvents: [pull_request_target]"),
        &[],
    )
    .unwrap();
    assert_eq!(lines_in(&found, "suppressed-null.yml"), vec![3]);
    assert_eq!(lines_in(&found, "suppressed-sequence.yml"), vec![4]);
    assert_eq!(lines_in(&found, "suppressed-mapping.yml"), vec![4]);
    assert_eq!(lines_in(&found, "suppressed-scalar.yml"), vec![2]);
}
