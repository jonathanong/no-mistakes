use super::super::checkout::check_source;
use super::super::RULE_ID;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

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

#[test]
fn saved_fixture_reports_only_unspecified_permissions() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/github-actions-checkout-credentials/fixture/assume-unspecified",
        ),
    );
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files: Vec<PathBuf> = [
        ".github/workflows/missing.yml",
        ".github/workflows/callable.yml",
        ".github/workflows/callable-write.yml",
        ".github/workflows/read.yml",
        ".github/workflows/write.yml",
        ".github/workflows/token.yml",
        ".github/workflows/persist-false.yml",
        ".github/workflows/null-permissions.yml",
        ".no-mistakes.yml",
    ]
    .into_iter()
    .map(|name| root.join(name))
    .collect();
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    let mut got: Vec<String> = findings
        .iter()
        .map(|finding| {
            let file = Path::new(&finding.file)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            format!("{file}:{}", finding.target.as_deref().unwrap_or(""))
        })
        .collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            "callable.yml:build".to_string(),
            "missing.yml:build".to_string(),
            "write.yml:build".to_string(),
        ]
    );
    for finding in &findings {
        let line = std::fs::read_to_string(root.join(&finding.file))
            .unwrap()
            .lines()
            .nth(finding.line - 1)
            .unwrap_or("")
            .to_string();
        assert!(
            line.contains("uses: actions/checkout@v4"),
            "{line} for {}",
            finding.file
        );
        if finding.file.ends_with("write.yml") {
            assert!(finding
                .message
                .contains("workflow permissions grant contents: write"));
        } else {
            assert!(finding.message.contains("assumeWriteWhenUnspecified"));
        }
    }
}

#[test]
fn unknown_option_is_rejected() {
    let config: NoMistakesConfig = serde_yaml::from_str(
        "rules:\n  - rule: github-actions-checkout-credentials\n    scope: repository\n    options:\n      nope: true\n",
    )
    .unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let error = super::super::check_with_files(&root, &config, &[])
        .unwrap_err()
        .to_string();
    assert!(error.contains(RULE_ID), "{error}");
    assert!(error.contains("nope"), "{error}");
}
