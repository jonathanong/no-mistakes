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

#[test]
fn block_comment_before_the_value_keeps_the_header_comment() {
    // Two block-scalar lines: the comment, then the value. The version comment
    // stays on the header, and the value is still the action line.
    let lines = [
        "- uses: | # v6.0.2",
        "    # still the block",
        "    actions/checkout@sha",
    ];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![3], "{classified:?}");
    assert!(classified.block_scalar[1] && classified.block_scalar[2]);
    assert_eq!(
        super::super::comment_at(&classified, &lines, 3),
        "- uses: | # v6.0.2"
    );
}

#[test]
fn root_with_is_not_an_action() {
    // No `jobs:` above this `with:`, so the nested uses is an input.
    let lines = ["with:", "  uses: actions/checkout@v4"];
    assert!(action_uses_lines(&lines).lines.is_empty(), "{lines:?}");
}

#[test]
fn a_word_after_uses_replaces_the_key() {
    // `uses extra:` is not the uses key. The following action is still checked.
    let lines = [
        "- uses extra: actions/checkout@v4",
        "- uses: actions/checkout@v4",
    ];
    assert_eq!(action_uses_lines(&lines).lines, vec![2], "{lines:?}");
}

#[test]
fn a_comment_under_an_anchor_keeps_the_step() {
    let lines = [
        "- &checkout",
        "  # version comes later",
        "  uses: actions/checkout@v4",
    ];
    assert_eq!(action_uses_lines(&lines).lines, vec![3], "{lines:?}");
}

#[test]
fn an_empty_anchor_name_does_not_hide_the_next_action() {
    // `& {` has no anchor name, so it is neither a key nor an anchor-only step.
    let lines = [
        "- uses: actions/checkout@v4",
        "- & {",
        "- uses: actions/setup-node@v4",
    ];
    assert_eq!(action_uses_lines(&lines).lines, vec![1, 3], "{lines:?}");
}
