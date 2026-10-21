use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    let mut parts = path.splitn(3, '/');
    let category = parts.next().unwrap_or(path);
    let sub = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        p = p.join(rest);
    }
    crate::codebase::ts_resolver::normalize_path(&p)
}

fn config_with_yaml(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn parent_segment_root_reports_missing_doc() {
    // `services/../agents` must resolve to `agents` before strip_prefix.
    let root = fixture("rules/required-local-docs/parent-segment");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let findings = check(&root, &config).unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.message.contains("agents/email") && finding.message.contains("missing")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("escapes")),
        "{findings:?}"
    );
}

#[test]
fn dot_prefix_root_still_matches() {
    let root = fixture("rules/required-local-docs/parent-segment");
    let config = config_with_yaml(
        "roots: [\"./agents\"]\nrequiredFile: README.md\ncodeExtensions: [mts]\ntestExcludePatterns: [\"*.test.*\"]",
    );
    let file = root.join("agents/email/index.mts");
    let findings = check_with_files(&root, &config, &[file]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("agents/email"), "{findings:?}");
    assert!(!findings[0].message.contains("escapes"));
}

#[test]
fn escaping_root_is_one_finding_and_skips_outside_files() {
    // The inside file would be a missing README if `..` were scanned as this
    // repo. The sibling fail fixture would be a finding if the parent were scanned.
    let root = fixture("rules/required-local-docs/escape");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let inside = root.join("agents/email/index.mts");
    let outside = fixture("rules/required-local-docs/fail").join("agents/email/index.mts");
    let opts: Options = config.rule_applications(RULE_ID)[0]
        .try_rule_options()
        .unwrap();
    let findings = scan(&root, &opts, &[inside, outside.clone()]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("escapes the repository"),
        "{findings:?}"
    );
    let outside_text = outside.display().to_string();
    assert!(
        findings.iter().all(|finding| {
            !finding.message.contains("missing")
                && !finding.file.contains("index.mts")
                && !finding.message.contains(&outside_text)
                && !finding.file.contains(&outside_text)
        }),
        "{findings:?}"
    );
    let discovered = check(&root, &config).unwrap();
    assert_eq!(discovered.len(), 1, "{discovered:?}");
    assert!(discovered[0].message.contains("escapes the repository"));
}
