use super::stored_runs::stored_run_blocks;
use super::{step_run_lines, step_uses_lines};

const MARKER: &str = "distinctive-run-literal";

fn with_run() -> String {
    format!(
        "\
on: push
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - run: |
          echo {MARKER}
          echo tail
"
    )
}

fn uses_only() -> String {
    "\
on: push
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
"
    .to_string()
}

fn stored_since(before: usize) -> usize {
    stored_run_blocks() - before
}

#[test]
fn uses_scan_does_not_store_run_blocks() {
    // `locate` is set here. Storing on that flag alone cloned every run block
    // and then discarded `scanner.runs`.
    let source = with_run();
    let before = stored_run_blocks();
    let uses = step_uses_lines(&source);
    assert!(!uses.is_empty());
    assert_eq!(stored_since(before), 0);
}

#[test]
fn run_scan_moves_each_block_once() {
    let source = with_run();
    let before = stored_run_blocks();
    let runs = step_run_lines(&source, false);
    assert!(runs
        .iter()
        .flat_map(|run| &run.lines)
        .any(|(_, text)| text.contains(MARKER)));
    assert_eq!(stored_since(before), 1);
}

#[test]
fn run_scan_without_a_run_block_stores_nothing() {
    let source = uses_only();
    let before = stored_run_blocks();
    let runs = step_run_lines(&source, false);
    assert!(runs.is_empty());
    assert_eq!(stored_since(before), 0);
}
