use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/tsconfig-file-coverage/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, yaml: &str) -> Vec<RuleFinding> {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    check_with_files(root, &config(yaml), &files).unwrap()
}

#[test]
fn uncovered_file_is_a_finding() {
    let findings = run(&fixture("fail"), "{}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("orphan.ts")
                && finding.message.contains("not covered by any tsconfig")),
        "{findings:?}"
    );
}

#[test]
fn included_sources_pass() {
    assert!(run(&fixture("pass"), "{}").is_empty());
}

#[test]
fn reasoned_allow_entry_covers_an_orphan() {
    let findings = run(
        &fixture("allow"),
        r#"
allow:
  - path: scripts/generate.ts
    reason: generated entrypoint kept outside the app program
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn auxiliary_config_is_excluded_from_membership() {
    let findings = run(
        &fixture("auxiliary"),
        r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: dependency-cruiser resolver config
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn empty_allow_reason_is_a_finding() {
    let findings = run(
        &fixture("allow"),
        r#"
allow:
  - path: scripts/generate.ts
    reason: ""
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("empty reason")),
        "{findings:?}"
    );
}

#[test]
fn stale_allow_entry_is_a_finding() {
    let findings = run(
        &fixture("pass"),
        r#"
allow:
  - path: missing.ts
    reason: leftover allow
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("stale")),
        "{findings:?}"
    );
}

#[test]
fn empty_auxiliary_reason_is_a_finding() {
    let findings = run(
        &fixture("auxiliary"),
        r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: ""
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("empty reason")),
        "{findings:?}"
    );
}

#[test]
fn auxiliary_config_may_not_declare_program_keys() {
    let findings = run(
        &fixture("pass"),
        r#"
auxiliaryConfigs:
  - path: tsconfig.json
    reason: misclassified compiler config
    requiredBasename: tsconfig.json
"#,
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must not declare files, include, exclude, or references")),
        "{findings:?}"
    );
}

#[test]
fn auxiliary_basename_must_match() {
    let findings = run(
        &fixture("pass"),
        r#"
auxiliaryConfigs:
  - path: tsconfig.json
    reason: wrong name
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("basename must be")),
        "{findings:?}"
    );
}

#[test]
fn missing_tsconfig_is_silent() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    let ts = root.join("index.ts");
    std::fs::write(&ts, "export {}\n").unwrap();
    let findings = check_with_files(&root, &config("{}"), &[ts]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn javascript_and_node_modules_are_skipped() {
    let root = fixture("fail");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("src/index.ts"),
        root.join("app.js"),
        root.join("node_modules/pkg/index.ts"),
    ];
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.file.contains("app.js")
                && !finding.file.contains("node_modules")),
        "{findings:?}"
    );
}

#[test]
fn option_defaults_are_empty() {
    let compiled = compile_options(&Options::default());
    assert!(compiled.allow.is_empty());
    assert!(compiled.auxiliary.is_empty());
    assert!(compiled.invalid.is_empty());
    let allow = ReasonedPath::default();
    assert!(allow.clone().path.is_empty());
    let auxiliary = AuxiliaryConfig::default();
    assert!(auxiliary.clone().path.is_empty());
}

#[test]
fn tsconfig_discovery_matches_basename_pattern() {
    assert!(scan::is_tsconfig_path("tsconfig.json"));
    assert!(scan::is_tsconfig_path("apps/web/tsconfig.app.json"));
    assert!(!scan::is_tsconfig_path("jsconfig.json"));
    assert!(!scan::is_tsconfig_path("node_modules/lib/tsconfig.json"));
    assert!(!scan::is_typescript_path("app.js", Path::new("app.js")));
    assert!(scan::is_typescript_path(
        "src/a.tsx",
        Path::new("src/a.tsx")
    ));
    assert!(!scan::is_typescript_path(
        "index.ts",
        Path::new("/tmp/node_modules/pkg/index.ts")
    ));
}

#[test]
fn normalize_rel_strips_dot_segments_and_rejects_escapes() {
    assert_eq!(normalize_rel("./src/foo.ts").as_deref(), Some("src/foo.ts"));
    assert_eq!(normalize_rel("").as_deref(), Some(""));
    assert_eq!(
        normalize_rel(r"scripts\generate.ts").as_deref(),
        Some("scripts/generate.ts")
    );
    assert_eq!(normalize_rel("/scripts/generate.ts"), None);
    assert_eq!(normalize_rel("../src/foo.ts"), None);
    assert_eq!(normalize_rel("src/../foo.ts"), None);
}

#[test]
fn source_include_keeps_root_tsconfig_membership() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        "{ \"files\": [\"src/index.ts\"] }\n",
    )
    .unwrap();
    std::fs::write(root.join("src/index.ts"), "export {}\n").unwrap();
    std::fs::write(root.join("src/extra.ts"), "export {}\n").unwrap();
    std::fs::write(root.join("scripts/scratch.ts"), "export {}\n").unwrap();
    let files = vec![
        root.join("tsconfig.json"),
        root.join("src/index.ts"),
        root.join("src/extra.ts"),
        root.join("scripts/scratch.ts"),
    ];
    let mut cfg = config("{}");
    cfg.rules[0].include = vec!["src/**/*.ts".into()];
    let findings = check_with_files(&root, &cfg, &files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.contains("extra.ts")
                && finding.message.contains("not covered by any tsconfig")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.file.contains("scratch.ts")),
        "{findings:?}"
    );
}

#[test]
fn absolute_allow_path_is_rejected_instead_of_rewritten() {
    let findings = run(
        &fixture("pass"),
        r#"
allow:
  - path: /src/index.ts
    reason: must not rewrite into the covered source
"#,
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must be a repository-relative path without parent traversals")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("stale")),
        "{findings:?}"
    );
}

#[test]
fn parent_traversal_allow_path_is_rejected() {
    let findings = run(
        &fixture("pass"),
        r#"
allow:
  - path: src/../src/index.ts
    reason: must not collapse traversals
"#,
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must be a repository-relative path without parent traversals")),
        "{findings:?}"
    );
}

#[test]
fn empty_allow_path_is_stale() {
    let findings = run(
        &fixture("pass"),
        r#"
allow:
  - path: ""
    reason: missing path
"#,
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("stale")),
        "{findings:?}"
    );
}

#[test]
fn whitespace_required_basename_uses_the_default() {
    let findings = run(
        &fixture("auxiliary"),
        r#"
requiredBasename: "  "
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: dependency-cruiser resolver config
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn auxiliary_only_inventory_leaves_sources_uncovered() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    std::fs::write(root.join("index.ts"), "export {}\n").unwrap();
    std::fs::write(root.join("tsconfig.dependency-cruiser.json"), "{}\n").unwrap();
    let files = vec![
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: resolver config
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file.contains("index.ts")
                && finding.message.contains("not covered by any tsconfig")),
        "{findings:?}"
    );
}

#[test]
fn missing_auxiliary_source_is_not_a_json_object() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    std::fs::write(root.join("index.ts"), "export {}\n").unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        "{ \"include\": [\"index.ts\"] }\n",
    )
    .unwrap();
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: missing from disk
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not a JSON object")),
        "{findings:?}"
    );
}

#[test]
fn auxiliary_array_json_is_not_an_object() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    std::fs::write(root.join("index.ts"), "export {}\n").unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        "{ \"include\": [\"index.ts\"] }\n",
    )
    .unwrap();
    std::fs::write(root.join("tsconfig.dependency-cruiser.json"), "[]\n").unwrap();
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: array document
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not a JSON object")),
        "{findings:?}"
    );
}

#[test]
fn tracked_non_tsconfig_auxiliary_is_still_read() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(&dir.path().canonicalize().unwrap());
    std::fs::write(root.join("index.ts"), "export {}\n").unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        "{ \"include\": [\"index.ts\"] }\n",
    )
    .unwrap();
    std::fs::write(root.join("foo.json"), "{ \"include\": [\"index.ts\"] }\n").unwrap();
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("foo.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: foo.json
    reason: misnamed helper
    requiredBasename: foo.json
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must not declare files, include, exclude, or references")),
        "{findings:?}"
    );
}

#[test]
fn invalid_rule_include_glob_is_a_configuration_error() {
    let root = fixture("pass");
    let mut cfg = config("{}");
    cfg.rules[0].include = vec!["[".into()];
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let error = check_with_files(&root, &cfg, &files).unwrap_err();
    assert!(error.to_string().contains("include"), "{error}");
}

#[test]
fn absolute_auxiliary_path_is_rejected() {
    let findings = run(
        &fixture("auxiliary"),
        r#"
auxiliaryConfigs:
  - path: /tsconfig.dependency-cruiser.json
    reason: must not rewrite
"#,
    );
    assert!(
        findings.iter().any(|finding| finding
            .message
            .contains("must be a repository-relative path without parent traversals")),
        "{findings:?}"
    );
}
