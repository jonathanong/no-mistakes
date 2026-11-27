use no_mistakes_rule_kernel::finding::RuleFinding;

fn findings(options: &str) -> Vec<RuleFinding> {
    let root = super::fixture("secrets-inherit");
    super::super::prepare::check_with_files(&root, &super::config(options), &[])
        .unwrap()
        .into_iter()
        .filter(|finding| !finding.file.contains("suppressed-"))
        .collect()
}

#[test]
fn reports_exact_inherit_on_local_quoted_and_remote_calls() {
    let found = findings("forbidSecretsInherit: true");
    assert_eq!(found.len(), 3);
    assert!(found
        .iter()
        .all(|finding| finding.file.ends_with("widgets.yml")));
    assert_eq!(
        found.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        vec![6, 18, 21]
    );
    assert!(found.iter().all(|finding| {
        finding.message
            == "forbidden secrets: inherit; map each secret explicitly on the reusable workflow call"
    }));
}

#[test]
fn omitted_and_false_leave_inherit_alone() {
    assert!(findings("{}").is_empty());
    assert!(findings("forbidSecretsInherit: false").is_empty());
}

#[test]
fn rejects_a_non_boolean_ban() {
    assert!(super::config("forbidSecretsInherit: [true]").rules[0]
        .try_rule_options::<super::super::Options>()
        .is_err());
}

#[test]
fn custom_messages_replace_the_inherit_diagnostic() {
    let root = super::fixture("secrets-inherit");
    let mut config = super::config("forbidSecretsInherit: true");
    config.rules[0].message = Some("Map each order secret explicitly.".to_owned());
    let found = super::super::prepare::check_with_files(&root, &config, &[])
        .unwrap()
        .into_iter()
        .filter(|finding| !finding.file.contains("suppressed-"))
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 3);
    assert!(found
        .iter()
        .all(|finding| finding.message == "Map each order secret explicitly."));
}
