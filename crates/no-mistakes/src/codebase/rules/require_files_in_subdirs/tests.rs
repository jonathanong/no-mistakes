use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture(path: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/require-files-in-subdirs/fixture")
            .join(path),
    )
}

// Output cases moved to cli_rra_snapshot_require_files_in_subdirs.

#[test]
fn no_op_when_packages_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_standalone_with_packages_discovers_files() {
    // Exercises lines 34-35 of check(): non-empty packages → discover_files.
    // Copy the saved tree into a temp dir outside this checkout. `git ls-files`
    // fails there, so discovery takes the non-git fallback walk this test used
    // before the fixture lived inside the repo. package.json is present, so
    // that walk produces no finding.
    let tmp = crate::test_support::materialize_saved_fixture(&fixture("email-package-only"));
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn absolute_package_root_is_used_directly() {
    // Exercises spec.root.is_absolute() branch (line 64).
    let root = fixture("email-package-only");
    let abs_queues = root.join("queues");
    let pkg = abs_queues.join("email/package.json");
    // Use absolute path for the package root
    let yaml = format!(
        "packages:\n  - root: {}\n    requiredFiles: [queues.mts]",
        abs_queues.display()
    );
    let config = config_with_rule(&yaml);
    let files = vec![pkg];
    let findings = check_with_files(&root, &config, &files).unwrap();
    // Should report missing queues.mts
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("queues.mts"));
}

#[test]
fn files_outside_root_are_skipped_in_first_level_subdirs() {
    // Exercises the strip_prefix error branch (line 145): files not under root are
    // skipped by first_level_subdirs.
    let bundle = fixture("outside-and-in");
    let root = bundle.join("root");
    let outside_file = bundle.join("outside/unrelated/something.ts");
    let pkg = root.join("queues/email/package.json");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    // Include both the in-root file and an out-of-root file
    let files = vec![pkg, outside_file];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "email subdir has package.json, should pass; outside file is skipped"
    );
}

#[test]
fn spec_root_itself_in_file_list_is_skipped_by_first_level_subdirs() {
    // Exercises line 149: when rel is empty (file == spec_root), components.next()
    // returns None and the iteration skips without recording a subdir.
    let root = fixture("spec-root-itself");
    let queues = root.join("queues");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    // Pass the spec root directory itself in the file list — strip_prefix returns ""
    let files = vec![queues];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "spec root dir in file list should produce no subdirs and no findings"
    );
}

#[test]
fn escaping_root_is_one_finding_and_skips_outside_files() {
    let root = fixture("escape");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let inside = root.join("agents/email/index.mts");
    let outside = fixture("fail").join("queues/email/package.json");
    let opts: Options = config.rule_applications(RULE_ID)[0]
        .try_rule_options()
        .unwrap();
    let findings = super::scan(&root, &config, &opts, &[inside.clone(), outside.clone()]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("escapes the repository"),
        "{findings:?}"
    );
    let outside_text = outside.display().to_string();
    assert!(
        findings.iter().all(|finding| {
            !finding.message.contains("missing required file")
                && !finding.file.contains("index.mts")
                && !finding.message.contains(&outside_text)
                && !finding.file.contains(&outside_text)
        }),
        "{findings:?}"
    );
    let checked = check_with_files(&root, &config, &[inside]).unwrap();
    assert_eq!(checked.len(), 1, "{checked:?}");
    assert!(checked[0].message.contains("escapes the repository"));
}
