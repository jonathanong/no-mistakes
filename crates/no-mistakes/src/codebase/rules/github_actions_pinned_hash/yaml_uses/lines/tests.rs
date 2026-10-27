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

    let next_line = [
        "jobs:",
        "  build:",
        "    strategy:",
        "      matrix:",
        "        include:",
        "          - uses:",
        "              ubuntu-latest",
        "    steps:",
        "      - { uses: actions/checkout@v4 }",
    ];
    let classified = action_uses_lines(&next_line);
    assert_eq!(classified.lines, vec![9], "{classified:?}");
    assert!(classified.rejected_uses[5], "{classified:?}");
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
fn hash_inside_a_plain_scalar_is_not_a_comment() {
    let line = "- { name: foo#bar, uses: actions/checkout@v4 }";
    assert_eq!(action_uses_lines(&[line]).lines, vec![1]);
    let comment = "- { name: foo #bar, uses: actions/checkout@v4 }";
    assert!(action_uses_lines(&[comment]).lines.is_empty());
}

#[test]
fn block_header_keeps_the_version_comment_line() {
    let lines = ["- uses: >- # v6.0.2", "    actions/checkout@sha"];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![2], "{classified:?}");
    assert!(classified.block_scalar[1]);
    assert_eq!(
        super::comment_at(&classified, &lines, 2),
        "- uses: >- # v6.0.2"
    );
}

#[test]
fn unicode_escape_in_the_key_is_uses() {
    let line = r#"- { "us\u0065s": actions/checkout@v4 }"#;
    assert_eq!(action_uses_lines(&[line]).lines, vec![1]);
    let short = r#"- { "us\x65s": actions/checkout@v4 }"#;
    assert_eq!(action_uses_lines(&[short]).lines, vec![1]);
    let long = r#"- { "us\U00000065s": actions/checkout@v4 }"#;
    assert_eq!(action_uses_lines(&[long]).lines, vec![1]);
    let broken = r#"- { "us\u00zzs": actions/checkout@v4 }"#;
    assert!(action_uses_lines(&[broken]).lines.is_empty());
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

#[test]
fn same_column_and_bare_dash_steps_are_action_lines() {
    // The dash shares a column with `steps`. The include field is not a step.
    let same = [
        "jobs:",
        "  build:",
        "    strategy:",
        "      matrix:",
        "        include:",
        "        - uses: ubuntu-latest",
        "    steps:",
        "    - uses: actions/checkout@v4",
        "    - uses: actions/setup-node@v4",
        "    timeout-minutes: 5",
    ];
    let classified = action_uses_lines(&same);
    assert_eq!(classified.lines, vec![8, 9], "{classified:?}");
    assert!(classified.rejected_uses[5], "{classified:?}");
    assert!(!classified.rejected_uses[7], "{classified:?}");

    let bare = [
        "jobs:",
        "  build:",
        "    steps:",
        "      -",
        "        uses: actions/checkout@v4",
        "      - # checkout",
        "        uses: actions/setup-node@v4",
    ];
    assert_eq!(action_uses_lines(&bare).lines, vec![5, 7], "{bare:?}");
}

#[test]
fn anchor_only_step_and_plain_apostrophe_keep_the_action() {
    let anchored = [
        "jobs:",
        "  build:",
        "    steps:",
        "      - &checkout",
        "        uses: actions/checkout@v4",
        "      - &same { uses: actions/setup-node@v4 }",
    ];
    assert_eq!(
        action_uses_lines(&anchored).lines,
        vec![5, 6],
        "{anchored:?}"
    );

    let named = ["- name: Don't fail", "- { uses: actions/checkout@v4 }"];
    assert_eq!(action_uses_lines(&named).lines, vec![2], "{named:?}");
    let quoted = ["- name: say \"hi\"", "- { uses: actions/checkout@v4 }"];
    assert_eq!(action_uses_lines(&quoted).lines, vec![2], "{quoted:?}");
}

#[test]
fn job_named_env_or_with_keeps_the_reusable_workflow() {
    let env_job = [
        "jobs:",
        "  env:",
        "    uses: owner/repo/.github/workflows/reuse.yml@v4",
    ];
    assert_eq!(action_uses_lines(&env_job).lines, vec![3], "{env_job:?}");
    let with_job = [
        "jobs:",
        "  with:",
        "    uses: owner/repo/.github/workflows/reuse.yml@v4",
    ];
    assert_eq!(action_uses_lines(&with_job).lines, vec![3], "{with_job:?}");
    let flow = ["jobs: { env: { uses: owner/repo/.github/workflows/reuse.yml@v4 } }"];
    assert_eq!(action_uses_lines(&flow).lines, vec![1], "{flow:?}");
    // A step `env` mapping is still an input, not an action line.
    let input = [
        "jobs:",
        "  build:",
        "    steps:",
        "      - uses: actions/checkout@v4",
        "        env:",
        "          USES: 1",
    ];
    assert_eq!(action_uses_lines(&input).lines, vec![4], "{input:?}");
}

#[test]
fn anchored_flow_sequence_repeats_each_action_line() {
    let lines = [
        "jobs:",
        "  one:",
        "    steps: &shared [{ uses: ./local }, { uses: actions/setup-node@v4 }]",
        "  two:",
        "    steps: *shared",
        "  three:",
        "    steps:",
        "      - { uses: actions/checkout@v4 }",
    ];
    assert_eq!(
        action_uses_lines(&lines).lines,
        vec![3, 3, 5, 5, 8],
        "{lines:?}"
    );
}

#[test]
fn job_named_steps_is_still_a_job() {
    let block = [
        "jobs:",
        "  steps:",
        "    uses: owner/repo/.github/workflows/reuse.yml@v4",
    ];
    let classified = action_uses_lines(&block);
    assert_eq!(classified.lines, vec![3], "{classified:?}");
    assert!(!classified.rejected_uses[2], "{classified:?}");
    let flow = ["jobs: { steps: { uses: owner/repo/.github/workflows/reuse.yml@v4 } }"];
    assert_eq!(action_uses_lines(&flow).lines, vec![1]);
}

#[test]
fn steps_alias_repeats_the_anchored_action_line() {
    let lines = [
        "jobs:",
        "  one:",
        "    steps: &shared",
        "      - uses: ./local",
        "  two:",
        "    steps: *shared",
        "  three:",
        "    steps:",
        "      - { uses: actions/checkout@v4 }",
    ];
    assert_eq!(action_uses_lines(&lines).lines, vec![4, 6, 9], "{lines:?}");
}

#[test]
fn plain_apostrophe_does_not_hide_the_version_comment() {
    let line = "- { name: Don't fail, uses: actions/checkout@sha } # v1";
    assert_eq!(super::super::version_comment(line), "# v1");
    let quoted = r#"- { name: "Don't", uses: actions/checkout@sha } # v6.0.2"#;
    assert_eq!(super::super::version_comment(quoted), "# v6.0.2");
}

#[test]
fn hash_inside_a_flow_value_keeps_the_version_comment() {
    let line = "- { name: foo#bar, uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd } # v6.0.2";
    assert_eq!(super::super::version_comment(line), "# v6.0.2");
    let embedded = "- { name: foo#bar, uses: actions/checkout@sha }";
    assert_eq!(super::super::version_comment(embedded), "");
    // Output cases moved to cli_gha_snapshot_pinned_hash.
}

// Output cases moved to cli_gha_snapshot_pinned_hash.

#[test]
fn anchor_key_on_the_dash_line_is_still_a_step() {
    let lines = [
        "jobs:",
        "  build:",
        "    steps:",
        "      - &checkout name: Checkout",
        "        uses: actions/checkout@v4",
    ];
    let classified = action_uses_lines(&lines);
    assert_eq!(classified.lines, vec![5], "{classified:?}");
    assert!(!classified.rejected_uses[4], "{classified:?}");
    // Output cases moved to cli_gha_snapshot_pinned_hash.
}

#[test]
fn nested_anchor_input_does_not_shift_the_pinned_step() {
    let lines = [
        "jobs:",
        "  build:",
        "    steps:",
        "      - &script",
        "        env:",
        "          uses: ./local",
        "        run: echo",
        "      - *script",
        "      - uses: actions/checkout@v4",
    ];
    assert_eq!(action_uses_lines(&lines).lines, vec![9], "{lines:?}");

    let flow = [
        "jobs:",
        "  build:",
        "    steps:",
        "      - &flow",
        "        env: { uses: ./local }",
        "      - *flow",
        "      - uses: actions/setup-node@v4",
    ];
    assert_eq!(action_uses_lines(&flow).lines, vec![7], "{flow:?}");
    // Output cases moved to cli_gha_snapshot_pinned_hash.
}

mod continued;
