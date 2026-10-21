use super::super::action_uses_lines;
use super::yaml_shape_findings;

#[test]
fn comment_between_uses_and_its_value_keeps_the_next_action() {
    let lines = [
        "- uses:",
        "",
        "  # local action",
        "  ./local",
        "- { uses: actions/checkout@v4 }",
    ];
    assert_eq!(action_uses_lines(&lines).lines, vec![4, 5], "{lines:?}");

    let findings = yaml_shape_findings("comment-before-value-fail.yml");
    let found: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(found, vec![10], "{findings:?}");
    assert!(
        findings[0].message.contains("actions/checkout@v4"),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("./local")),
        "{findings:?}"
    );
}

#[test]
fn folded_quote_reads_the_comment_on_the_closing_line() {
    let lines = [
        r#"- { uses: "actions/checkout@\"#,
        r#"de0fac2e4500dabe0009e67214ff5f5447ce83dd" } # v6.0.2"#,
    ];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![1], "{classified:?}");
    assert!(
        super::super::comment_at(&classified, &lines, 1).contains("# v6.0.2"),
        "{classified:?}"
    );

    let findings = yaml_shape_findings("folded-quote-comment-pass.yml");
    assert!(findings.is_empty(), "{findings:?}");
}
