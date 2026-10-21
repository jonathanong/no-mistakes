use super::action_uses_lines;

#[test]
fn commented_uses_is_one_action_line() {
    let line = "- uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2";
    assert_eq!(action_uses_lines(&[line]).lines, vec![1]);
    let quoted = action_uses_lines(&["- uses: \"actions/checkout@v4\""]);
    assert_eq!(quoted.lines, vec![1]);
}

#[test]
fn flow_uses_on_one_line_is_an_action_line() {
    let classified = action_uses_lines(&["- { uses: actions/checkout@v4 }"]);
    assert_eq!(
        classified.lines,
        vec![1],
        "{:?}",
        classified.flow_input_at_start
    );
}

#[test]
fn same_line_flow_steps_keep_every_uses() {
    let line = "steps: [{ uses: actions/checkout@v4 }, { uses: actions/setup-node@v4 }]";
    let classified = action_uses_lines(&[line]);
    assert_eq!(classified.lines, vec![1, 1], "{classified:?}");
}

#[test]
fn block_script_is_not_an_action_line() {
    let lines = [
        "- run: |",
        "    echo { uses: actions/checkout@v4 }",
        "- uses: actions/checkout@v4",
    ];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![3], "{classified:?}");
    assert!(classified.block_scalar[1]);
}

#[test]
fn block_uses_value_is_the_next_line() {
    let lines = ["- uses: |", "    actions/checkout@v4"];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![2], "{classified:?}");
}

#[test]
fn matrix_include_uses_is_not_an_action_line() {
    // A flow or block matrix field named uses must not take the step's line.
    let flow = [
        "on: [push]",
        "jobs:",
        "  build:",
        "    strategy:",
        "      matrix: { include: [{ uses: ubuntu-latest }] } # v6.0.2",
        "    steps:",
        "      - { uses: actions/checkout@v4 }",
    ];
    let classified = action_uses_lines(&flow);
    assert_eq!(classified.lines, vec![7], "{classified:?}");
    assert!(classified.rejected_uses[4], "{classified:?}");

    let block = [
        "jobs:",
        "  build:",
        "    uses: org/repo/.github/workflows/build.yml@v4",
        "    strategy:",
        "      matrix:",
        "        include:",
        "          - uses: ubuntu-latest",
        "    steps:",
        "      - uses: actions/checkout@v4",
    ];
    let classified = action_uses_lines(&block);
    assert_eq!(classified.lines, vec![3, 9], "{classified:?}");
    assert!(classified.rejected_uses[6], "{classified:?}");
}

#[test]
fn flow_closers_count_alias_and_tight_uses() {
    // `}` and a missing space still close the action scalar once.
    let alias = action_uses_lines(&["- { uses: *checkout }"]);
    assert_eq!(alias.lines, vec![1], "{alias:?}");
    let tight = action_uses_lines(&["- { uses: actions/checkout@v4}"]);
    assert_eq!(tight.lines, vec![1], "{tight:?}");
    let root = action_uses_lines(&["[{ uses: actions/checkout@v4 }]"]);
    assert_eq!(root.lines, vec![1], "{root:?}");
}

#[test]
fn escaped_double_quote_keeps_the_action_line() {
    let line = r#"- { name: "foo\"bar", uses: actions/checkout@v4 }"#;
    let classified = action_uses_lines(&[line]);
    assert_eq!(classified.lines, vec![1], "{classified:?}");

    let key = r#"- { "foo\"bar": true, uses: actions/checkout@v4 }"#;
    let classified = action_uses_lines(&[key]);
    assert_eq!(classified.lines, vec![1], "{classified:?}");

    let value = r#"- { uses: "actions/checkout@v4\"x" }"#;
    let classified = action_uses_lines(&[value]);
    assert_eq!(classified.lines, vec![1], "{classified:?}");

    // A trailing `\` escapes the line break. The next line's quote still closes.
    let continued = action_uses_lines(&[r#"- { name: "foo\"#, r#"", uses: actions/checkout@v4 }"#]);
    assert_eq!(continued.lines, vec![2], "{continued:?}");
}

#[test]
fn commented_block_header_is_not_an_action_line() {
    let lines = [
        "- run: | # explain",
        "    echo { uses: actions/checkout@v4 }",
        "- uses: actions/checkout@v4",
    ];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![3], "{classified:?}");
    assert!(classified.block_scalar[1]);

    let folded = [
        "- run: >2 # explain",
        "    echo { uses: actions/checkout@v4 }",
        "- uses: actions/checkout@v4",
    ];
    let classified = action_uses_lines(&folded);
    assert_eq!(classified.lines, vec![3], "{classified:?}");
    assert!(classified.block_scalar[1]);

    let uses = ["- uses: | # kept", "    actions/checkout@v4"];
    let classified = action_uses_lines(&uses);
    assert_eq!(classified.lines, vec![2], "{classified:?}");
}

#[test]
fn space_before_colon_keeps_the_uses_key() {
    let line = "- uses : actions/checkout@v4";
    assert_eq!(action_uses_lines(&[line]).lines, vec![1]);
    let flow = action_uses_lines(&["- { uses : actions/checkout@v4 }"]);
    assert_eq!(flow.lines, vec![1], "{flow:?}");
}

#[test]
fn mapping_alias_step_is_an_action_line() {
    let flow = [
        "jobs:",
        "  call:",
        "    strategy:",
        "      matrix:",
        "        include:",
        "          - &shared { uses: actions/checkout@v4 }",
        "    steps: [*shared]",
    ];
    assert_eq!(action_uses_lines(&flow).lines, vec![7], "{flow:?}");

    let block = [
        "jobs:",
        "  call:",
        "    strategy:",
        "      matrix:",
        "        include:",
        "          - &plain",
        "            uses: actions/checkout@v3",
        "    steps:",
        "      - *plain",
    ];
    assert_eq!(action_uses_lines(&block).lines, vec![9], "{block:?}");
}

#[test]
fn bare_dash_shapes_are_not_action_lines() {
    let bare = action_uses_lines(&["-"]);
    assert!(bare.lines.is_empty(), "{bare:?}");
    // `-uses` is not a sequence item. A root key is not a job or step.
    let stuck = action_uses_lines(&["-uses: actions/checkout@v4"]);
    assert!(stuck.lines.is_empty(), "{stuck:?}");
    let unclosed = action_uses_lines(&["- \"uses"]);
    assert!(unclosed.lines.is_empty(), "{unclosed:?}");
}

#[test]
fn indented_root_uses_is_not_rejected() {
    // The reusable-workflow regex fixture is only this line. It is not a
    // nested matrix field, so the line scan must keep the finding.
    let line = "    uses: org/repo/.github/workflows/build.yml@main";
    let classified = action_uses_lines(&[line]);
    assert!(classified.lines.is_empty(), "{classified:?}");
    assert!(!classified.rejected_uses[0], "{classified:?}");
}
