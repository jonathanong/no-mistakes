use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn saved_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency")
            .join(name),
    )
}

fn path_regex_config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[cfg(unix)]
#[test]
fn path_regex_capture_reads_directory_target_symlinks_from_inventory() {
    let root = saved_fixture("path-regex-directory-symlink");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let link = root.join("alpha");
    assert!(
        sources.inventory().paths().contains(&link),
        "snapshot inventory must keep the directory-target symlink"
    );
    assert!(!sources.inventory().target_file_paths().contains(&link));

    // Omit the symlink from the check file slice the way `check` does.
    let findings = check_with_files_and_sources(
        &root,
        &path_regex_config(
            r#"
sets:
  - name: fileSet
    kind: path-regex-capture
    pattern: "^(?<value>alpha)\\.txt$"
  - name: linkSet
    kind: path-regex-capture
    pattern: "^(?<value>alpha)$"
comparisons:
  - left: fileSet
    right: linkSet
    mode: equal-set
"#,
        ),
        &[root.join("alpha.txt")],
        &sources,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[cfg(unix)]
#[test]
fn path_regex_capture_honors_exclude_on_inventory_path_entries() {
    let root = saved_fixture("path-regex-skill-symlinks");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let mut config = path_regex_config(
        r#"
sets:
  - name: skillFiles
    kind: path-regex-capture
    pattern: "^\\.agents/skills/(?<value>[^/]+)/SKILL\\.md$"
  - name: skillLinks
    kind: path-regex-capture
    pattern: "^\\.claude/skills/(?<value>[^/]+)$"
comparisons:
  - left: skillFiles
    right: skillLinks
    mode: equal-set
"#,
    );
    config.rules[0].exclude = vec![".claude/**".to_string()];

    let files = sources.inventory().target_file_paths();
    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();

    assert!(
        findings.iter().any(|finding| {
            finding
                .message
                .contains("skillFiles contains `agent-workflow`")
                && finding.message.contains("skillLinks does not")
        }),
        "{findings:?}"
    );
}

#[cfg(unix)]
#[test]
fn path_regex_capture_matches_broken_and_outside_root_symlink_paths() {
    let broken_root = saved_fixture("path-regex-broken-symlink");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&broken_root);
    let sources = snapshot.source_store_for(&broken_root);
    let findings = check_with_files_and_sources(
        &broken_root,
        &path_regex_config(
            r#"
sets:
  - name: fileSet
    kind: path-regex-capture
    pattern: "^(?<value>delta)\\.txt$"
  - name: linkSet
    kind: path-regex-capture
    pattern: "^(?<value>delta)$"
comparisons:
  - left: fileSet
    right: linkSet
    mode: equal-set
"#,
        ),
        &[broken_root.join("delta.txt")],
        &sources,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let outside_root = saved_fixture("path-regex-directory-symlink");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&outside_root);
    let sources = snapshot.source_store_for(&outside_root);
    let findings = check_with_files_and_sources(
        &outside_root,
        &path_regex_config(
            r#"
sets:
  - name: fileSet
    kind: path-regex-capture
    pattern: "^(?<value>gamma)\\.txt$"
  - name: linkSet
    kind: path-regex-capture
    pattern: "^(?<value>gamma)$"
comparisons:
  - left: fileSet
    right: linkSet
    mode: equal-set
"#,
        ),
        &[outside_root.join("gamma.txt")],
        &sources,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[cfg(unix)]
#[test]
fn path_regex_capture_does_not_reopen_regular_files_omitted_from_the_work_list() {
    let root = saved_fixture("path-regex-directory-symlink");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let findings = check_with_files_and_sources(
        &root,
        &path_regex_config(
            r#"
sets:
  - name: kept
    kind: path-regex-capture
    pattern: "^(?<value>alpha)\\.txt$"
  - name: omitted
    kind: path-regex-capture
    pattern: "^(?<value>beta)\\.txt$"
comparisons:
  - left: kept
    right: omitted
    mode: equal-set
"#,
        ),
        &[root.join("alpha.txt")],
        &sources,
    )
    .unwrap();

    assert!(
        findings.iter().any(|finding| {
            finding.message.contains("kept contains `alpha`")
                && finding.message.contains("omitted does not")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("`beta`")),
        "{findings:?}"
    );
}
