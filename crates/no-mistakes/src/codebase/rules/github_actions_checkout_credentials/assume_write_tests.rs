use super::super::checkout::check_source;
use crate::codebase::rules::RuleFinding;

#[path = "assume_write_fixture_tests.rs"]
mod assume_write_fixture_tests;

const WORKFLOW: &str = ".github/workflows/ci.yml";

fn findings(source: &str, assume: bool) -> Vec<RuleFinding> {
    check_source(WORKFLOW, source, assume)
}

#[test]
fn missing_permissions_stay_silent_unless_the_option_is_set() {
    let push = "\
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
";
    assert!(findings(push, false).is_empty());
    let flagged = findings(push, true);
    assert_eq!(flagged.len(), 1, "{flagged:?}");
    assert!(flagged[0].message.contains("assumeWriteWhenUnspecified"));
    assert_eq!(flagged[0].line, 6);

    let callable = "\
on: workflow_call
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
";
    assert!(findings(callable, false).is_empty());
    let assumed = findings(callable, true);
    assert_eq!(assumed.len(), 1, "{assumed:?}");
    assert!(assumed[0].message.contains("assumeWriteWhenUnspecified"));
}

#[test]
fn the_option_does_not_override_an_explicit_grant_or_token() {
    let callable_write = "\
on: workflow_call
permissions:
  contents: write
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
";
    assert!(findings(callable_write, true).is_empty());

    let read = "\
on: push
permissions:
  contents: read
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
";
    assert!(findings(read, true).is_empty());

    let token = "\
on: push
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
        with:
          token: ${{ secrets.RELEASE_PAT }}
";
    assert!(findings(token, true).is_empty());

    let proven = findings(
        "\
on: push
permissions:
  contents: write
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
",
        true,
    );
    assert_eq!(proven.len(), 1, "{proven:?}");
    assert!(proven[0]
        .message
        .contains("workflow permissions grant contents: write"));
    assert!(!proven[0].message.contains("assumeWriteWhenUnspecified"));
}
