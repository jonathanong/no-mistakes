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

fn assume_unspecified_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/github-actions-checkout-credentials/fixture/assume-unspecified",
    ))
}

fn saved_config(root: &Path, name: &str) -> NoMistakesConfig {
    serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap()
}

fn assume_unspecified_files(root: &Path) -> Vec<PathBuf> {
    [
        ".github/workflows/missing.yml",
        ".github/workflows/callable.yml",
        ".github/workflows/callable-write.yml",
        ".github/workflows/read.yml",
        ".github/workflows/write.yml",
        ".github/workflows/token.yml",
        ".github/workflows/persist-false.yml",
        ".github/workflows/null-permissions.yml",
        ".github/workflows/suppressed.yml",
        ".no-mistakes.yml",
        ".no-mistakes.off.yml",
    ]
    .into_iter()
    .map(|name| root.join(name))
    .collect()
}

fn finding_ids(findings: &[RuleFinding]) -> Vec<String> {
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
    got
}

fn assert_checkout_lines(root: &Path, findings: &[RuleFinding]) {
    for finding in findings {
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
    }
}

#[test]
fn saved_fixture_reports_only_unspecified_permissions() {
    let root = assume_unspecified_root();
    let files = assume_unspecified_files(&root);
    let config = saved_config(&root, ".no-mistakes.yml");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(
        finding_ids(&findings),
        vec![
            "callable.yml:build".to_string(),
            "missing.yml:build".to_string(),
            "suppressed.yml:build".to_string(),
            "write.yml:build".to_string(),
        ]
    );
    assert_checkout_lines(&root, &findings);
    for finding in &findings {
        if finding.file.ends_with("write.yml") {
            assert!(finding
                .message
                .contains("workflow permissions grant contents: write"));
            assert!(!finding.message.contains("assumeWriteWhenUnspecified"));
        } else {
            assert!(finding.message.contains("assumeWriteWhenUnspecified"));
        }
    }

    let off = saved_config(&root, ".no-mistakes.off.yml");
    let off_findings = super::super::check_with_files(&root, &off, &files).unwrap();
    assert_eq!(
        finding_ids(&off_findings),
        vec!["write.yml:build".to_string()]
    );
    assert_checkout_lines(&root, &off_findings);
    assert!(off_findings[0]
        .message
        .contains("workflow permissions grant contents: write"));
    assert!(!off_findings[0]
        .message
        .contains("assumeWriteWhenUnspecified"));

    let visible = crate::codebase::ts_source::discover_files(&root, &[]);
    assert!(
        visible.iter().any(|path| path.ends_with("suppressed.yml")),
        "filtered run must see suppressed.yml, got {visible:?}"
    );
    let filtered =
        crate::codebase::rules::run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    assert_eq!(
        finding_ids(&filtered),
        vec![
            "callable.yml:build".to_string(),
            "missing.yml:build".to_string(),
            "write.yml:build".to_string(),
        ]
    );
    assert!(filtered
        .iter()
        .all(|finding| !finding.file.ends_with("suppressed.yml")));
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
