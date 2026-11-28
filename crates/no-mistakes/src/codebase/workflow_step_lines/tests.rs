use super::*;

fn value<'a>(step: &'a Step, name: &str) -> Option<&'a str> {
    step.fields
        .iter()
        .rev()
        .find(|field| field.name == name)
        .map(|field| field.value.as_str())
}

#[test]
fn first_key_line_uses_block_keys_and_step_fallback() {
    let source = "\
jobs:\n  \
  unit:\n    \
    steps:\n      \
      - uses: actions/cache@v4\n        \
        with:\n          \
          action-timeout-s: 1\n";
    assert_eq!(
        first_key_line(source, "unit", 0, &["action-timeout-s", "uses"]),
        6
    );
    assert_eq!(first_key_line(source, "missing", 0, &["uses"]), 1);
    assert_eq!(first_key_line(source, "unit", 3, &["uses"]), 1);
    let flow = "\
jobs:\n  \
  unit:\n    \
    steps:\n      \
      - uses: actions/cache@v4\n        \
        with: { action-timeout-s: 1 }\n";
    assert_eq!(
        first_key_line(flow, "unit", 0, &["action-timeout-s", "uses", "with"]),
        4
    );
    let composite = "\
runs:\n  \
  using: composite\n  \
  steps:\n    \
    - uses: actions/cache@v4\n";
    assert_eq!(first_key_line(composite, "(composite)", 0, &["uses"]), 4);
}

#[test]
fn flow_scalars_accept_numbers_bools_sequences_and_reject_the_rest() {
    let mut step = Step::blank(4);
    flow::fill_flow(
        &mut step,
        4,
        "{ id: lock, uses: actions/cache, run: echo, with: { path: .cache/widgets, key: 7, restore-keys: [widgets-, orders-] } }",
    );
    assert_eq!(value(&step, "id"), Some("lock"));
    assert_eq!(value(&step, "uses"), Some("actions/cache"));
    assert_eq!(value(&step, "run"), Some("echo"));
    assert_eq!(value(&step, "path"), Some(".cache/widgets"));
    assert_eq!(value(&step, "key"), Some("7"));
    assert_eq!(value(&step, "restore-keys"), Some("widgets-\norders-"));

    let mut skipped = Step::blank(1);
    flow::fill_flow(&mut skipped, 1, "{");
    flow::fill_flow(&mut skipped, 1, "[1, 2]");
    flow::fill_flow(&mut skipped, 1, "{ uses: actions/cache, with: plain }");
    assert!(value(&skipped, "path").is_none());
    assert_eq!(value(&skipped, "uses"), Some("actions/cache"));

    let mut with = Step::blank(2);
    flow::fill_with_text(&mut with, 2, "{");
    flow::fill_with_text(&mut with, 2, "[]");
    flow::fill_with_text(
        &mut with,
        2,
        "{ path: true, key: [], restore-keys: { nested: 1 } }",
    );
    assert_eq!(value(&with, "path"), Some("true"));
    assert!(value(&with, "key").is_none());
    assert!(value(&with, "restore-keys").is_none());
    flow::fill_with_text(&mut with, 2, "{ key: [1, widgets], path: null }");
    assert_eq!(value(&with, "key"), Some("widgets"));
    assert_eq!(value(&with, "path"), Some("true"));
}

#[test]
fn yaml_cursors_reject_keys_that_are_not_plain() {
    assert_eq!(
        flow::split_kv("uses: actions/cache"),
        Some(("uses", "actions/cache"))
    );
    assert!(flow::split_kv("no-colon").is_none());
    assert!(flow::split_kv(": empty").is_none());
    assert!(flow::split_kv("bad key: x").is_none());
    assert_eq!(
        flow::list_item("  - uses: actions/cache"),
        Some((2, "uses: actions/cache"))
    );
    assert!(flow::list_item("uses: actions/cache").is_none());
    assert!(flow::list_item("-uses").is_none());
    assert!(flow::mapping_key("  runs/using: composite").is_some());
    assert!(flow::mapping_key("  - uses: actions/cache").is_none());
    assert!(flow::mapping_key("  # comment").is_none());
    assert!(flow::mapping_key("  not a key").is_none());
    assert!(flow::mapping_key("  bad key: x").is_none());
    assert_eq!(flow::comment_value(" widgets # note"), "widgets");
    assert_eq!(flow::comment_value(" # note"), "");
    assert_eq!(
        flow::comment_value(" \"widgets # not\" "),
        "\"widgets # not\""
    );
    assert_eq!(flow::comment_value(" |"), "|");
    assert_eq!(
        flow::comment_value(" { path: .cache/widgets }"),
        "{ path: .cache/widgets }"
    );
    assert_eq!(flow::comment_value(" [widgets]"), "[widgets]");
    assert!(flow::block_header("|"));
    assert!(flow::block_header(">+2"));
    assert!(flow::block_header(">#"));
    assert!(!flow::block_header(""));
    assert!(!flow::block_header("|bad"));
    assert!(!flow::block_header("plain"));
    assert_eq!(flow::leading("   key"), 3);
}

#[test]
fn edits_with_no_open_step_are_ignored() {
    let mut walker = cursor::Walker::new(false);
    assert!(!scalar::consume_scalar(&mut walker, "  widgets"));
    scalar::finish_scalar(&mut walker);
    scalar::assign(&mut walker, "run", 2, 0, "echo\nwidgets.lock");
    scalar::assign(&mut walker, "key", 2, 0, "\"widgets\"");
    scalar::open_with(&mut walker, 2, 0, "inline");
    scalar::open_with(&mut walker, 2, 0, "{ path: .cache/widgets }");
    assert!(!walker.in_with);
    scalar::open_with(&mut walker, 2, 0, "");
    assert!(walker.in_with);
    walker.seq = Some(scalar::Seq {
        name: "path".to_string(),
        indent: 2,
        line: 2,
    });
    assert!(scalar::consume_seq(&mut walker, 3, "    - .cache/widgets"));
    assert!(!scalar::consume_seq(&mut walker, 4, "  - .cache/orders"));
    assert!(walker.seq.is_none());
    walker.seq = Some(scalar::Seq {
        name: "restore-keys".to_string(),
        indent: 2,
        line: 2,
    });
    assert!(!scalar::consume_seq(&mut walker, 5, "key: widgets"));
    assert!(walker.seq.is_none());
    assert!(walker.jobs.is_empty());
}

#[test]
fn stray_list_items_do_not_open_steps() {
    let source = "---\n\nsteps:\njobs:\n  build:\n    - not-a-step\n    steps:\n      name: early\n      - uses: actions/checkout@v4\n        - nested\n      key: sibling\n- stray\n...\n";
    let jobs = walk_steps(source, false);
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name, "build");
    assert_eq!(jobs[0].steps.len(), 1);
    assert_eq!(
        value(&jobs[0].steps[0], "uses"),
        Some("actions/checkout@v4")
    );
}

#[test]
fn quoted_scalars_keep_escapes_blank_lines_and_dedents() {
    assert!(!cont::quote_closed(""));
    assert!(!cont::quote_closed("widgets"));
    assert!(cont::quote_closed("\"a\\nb\""));
    assert!(cont::quote_closed("'it''s'"));
    assert_eq!(cont::unwrap_quotes("\"a\\nb\""), "anb");
    assert_eq!(cont::unwrap_quotes("'it''s'"), "it's");
    assert_eq!(cont::unwrap_quotes("\"open"), "\"open");

    let source = "\
jobs:\n  \
  build:\n    \
    steps:\n      \
      - run: \"echo\n          \
        still\n          \
        widgets\\nlock\"\n      \
      - run: \"echo\n\n          \
        widgets\"\n      \
      - run: 'it''s\n          \
        fine'\n      \
      - run: \"echo widgets\n        \
        uses: actions/checkout@v4\n";
    let jobs = walk_steps(source, false);
    assert_eq!(jobs.len(), 1, "{jobs:?}");
    let steps = &jobs[0].steps;
    assert_eq!(steps.len(), 4, "{steps:?}");
    assert_eq!(value(&steps[0], "run"), Some("echo\nstill\nwidgetsnlock"));
    assert_eq!(value(&steps[1], "run"), Some("echo\n\nwidgets"));
    assert_eq!(value(&steps[2], "run"), Some("it's\nfine"));
    assert_eq!(value(&steps[3], "run"), Some("\"echo widgets"));
    assert_eq!(value(&steps[3], "uses"), Some("actions/checkout@v4"));
}

#[test]
fn empty_sequence_item_appends_without_a_blank_line() {
    let source = "\
jobs:\n  \
  build:\n    \
    steps:\n      \
      - uses: actions/cache@v4\n        \
        with:\n          \
          path:\n            \
            - \"\"\n            \
            - .cache/widgets\n";
    let jobs = walk_steps(source, false);
    assert_eq!(value(&jobs[0].steps[0], "path"), Some(".cache/widgets"));
}
