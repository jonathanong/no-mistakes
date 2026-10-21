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
