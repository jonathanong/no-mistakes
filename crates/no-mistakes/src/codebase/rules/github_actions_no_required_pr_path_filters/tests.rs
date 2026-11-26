use super::{collect_config_errors, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

fn parse_config(yaml: &str) -> NoMistakesConfig {
    serde_yaml::from_str(yaml).expect("parse rule config")
}

#[test]
fn collect_config_errors_requires_non_empty_workflows_when_enabled() {
    for yaml in [
        r#"
rules:
  - rule: github-actions-no-required-pr-path-filters
    scope: repository
"#,
        r#"
rules:
  - rule: github-actions-no-required-pr-path-filters
    scope: repository
    options:
      workflows: []
"#,
    ] {
        let config = parse_config(yaml);
        let mut errors = Vec::new();
        collect_config_errors(Path::new("/repo"), None, &config, &mut errors);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains(RULE_ID), "{errors:?}");
        assert!(errors[0].contains("options.workflows"), "{errors:?}");
        assert!(errors[0].contains("[]"), "{errors:?}");
    }
}

#[test]
fn collect_config_errors_skips_disabled_rule() {
    let config = parse_config(
        r#"
rules:
  - rule: github-actions-no-required-pr-path-filters
    enabled: false
    scope: repository
"#,
    );
    let mut errors = Vec::new();
    collect_config_errors(Path::new("/repo"), None, &config, &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn collect_config_errors_reports_filter_schema_and_non_literal_errors() {
    let config = parse_config(
        r#"
rules:
  - rule: github-actions-no-required-pr-path-filters
    scope: repository
    include: ["[unclosed"]
    options:
      workflows: true
  - rule: github-actions-no-required-pr-path-filters
    scope: repository
    options:
      workflows:
        - .github/workflows/*.yml
        - scripts/ci.yml
"#,
    );
    let mut errors = Vec::new();
    collect_config_errors(Path::new("/repo"), None, &config, &mut errors);
    let joined = errors.join("\n");
    assert!(joined.contains("[unclosed"), "{joined}");
    assert!(joined.contains("options.workflows"), "{joined}");
    assert!(joined.contains(".github/workflows/*.yml"), "{joined}");
    assert!(joined.contains("scripts/ci.yml"), "{joined}");
}
