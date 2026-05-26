use super::doc_section::scan_doc_section;
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

fn config_with_yaml(rule_id: &str, yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: rule_id.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

// ── is_code_file unit tests ──────────────────────────────────────────────────

fn default_excl_globs() -> GlobSet {
    build_exclude_globs(DEFAULT_TEST_EXCLUDE)
}

#[test]
fn code_file_with_matching_extension_passes() {
    let globs = default_excl_globs();
    assert!(is_code_file(
        Path::new(""),
        Path::new("agents/email/index.mts"),
        DEFAULT_CODE_EXTENSIONS,
        DEFAULT_TEST_EXCLUDE,
        &globs
    ));
}

#[test]
fn non_code_extension_fails() {
    let globs = default_excl_globs();
    assert!(!is_code_file(
        Path::new(""),
        Path::new("agents/email/style.css"),
        DEFAULT_CODE_EXTENSIONS,
        DEFAULT_TEST_EXCLUDE,
        &globs
    ));
}

#[test]
fn test_file_by_name_pattern_excluded() {
    let globs = default_excl_globs();
    assert!(!is_code_file(
        Path::new(""),
        Path::new("agents/email/index.test.mts"),
        DEFAULT_CODE_EXTENSIONS,
        DEFAULT_TEST_EXCLUDE,
        &globs
    ));
}

#[test]
fn test_dir_component_excluded() {
    let globs = default_excl_globs();
    assert!(!is_code_file(
        Path::new(""),
        Path::new("agents/email/__tests__/index.mts"),
        DEFAULT_CODE_EXTENSIONS,
        DEFAULT_TEST_EXCLUDE,
        &globs
    ));
}

#[test]
fn spec_file_excluded() {
    let globs = default_excl_globs();
    assert!(!is_code_file(
        Path::new(""),
        Path::new("agents/email/foo.spec.ts"),
        DEFAULT_CODE_EXTENSIONS,
        DEFAULT_TEST_EXCLUDE,
        &globs
    ));
}

#[test]
fn path_exclude_pattern_excluded() {
    let globs = build_exclude_globs(&["agents/generated/**"]);
    assert!(!is_code_file(
        Path::new(""),
        Path::new("agents/generated/index.mts"),
        DEFAULT_CODE_EXTENSIONS,
        &[],
        &globs
    ));
}

// ── scan unit tests ──────────────────────────────────────────────────────────

fn opts_with_roots(roots: &[&str]) -> Options {
    Options {
        roots: roots.iter().map(PathBuf::from).collect(),
        required_file: "README.md".to_string(),
        code_extensions: vec!["mts".to_string(), "ts".to_string()],
        test_exclude_patterns: vec!["*.test.*".to_string(), "__tests__".to_string()],
    }
}

#[test]
fn scan_empty_roots_returns_empty() {
    let opts = Options::default();
    assert!(scan(Path::new("/tmp"), &opts, &[]).is_empty());
}

#[test]
fn scan_subdir_with_code_and_readme_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("agents/email");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.mts"), "").unwrap();
    std::fs::write(dir.join("README.md"), "# hi").unwrap();
    let files: Vec<PathBuf> = vec![dir.join("index.mts"), dir.join("README.md")];
    let opts = opts_with_roots(&["agents"]);
    let findings = scan(root, &opts, &files);
    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
}

#[test]
fn scan_subdir_without_readme_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("agents/email");
    std::fs::create_dir_all(&dir).unwrap();
    let code_file = dir.join("index.mts");
    std::fs::write(&code_file, "").unwrap();
    let files = vec![code_file];
    let opts = opts_with_roots(&["agents"]);
    let findings = scan(root, &opts, &files);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing README.md"));
}

#[test]
fn scan_skips_files_directly_under_root() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let code_file = dir.join("top.mts");
    std::fs::write(&code_file, "").unwrap();
    let files = vec![code_file];
    let opts = opts_with_roots(&["agents"]);
    let findings = scan(root, &opts, &files);
    assert!(
        findings.is_empty(),
        "direct root children should not require docs"
    );
}

#[test]
fn scan_skips_test_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("agents/email/__tests__");
    std::fs::create_dir_all(&dir).unwrap();
    let test_file = dir.join("index.test.mts");
    std::fs::write(&test_file, "").unwrap();
    let files = vec![test_file];
    let opts = opts_with_roots(&["agents"]);
    let findings = scan(root, &opts, &files);
    assert!(
        findings.is_empty(),
        "test files should not trigger missing docs"
    );
}

// ── scan_doc_section unit tests ──────────────────────────────────────────────

#[test]
fn doc_section_passes_when_heading_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let readme = root.join("agents/email/README.md");
    std::fs::create_dir_all(readme.parent().unwrap()).unwrap();
    std::fs::write(&readme, "# Email\n\n## Performance\n\nFast.\n").unwrap();
    let opts = DocSectionOptions {
        glob: "agents/*/README.md".to_string(),
        required_heading: "## Performance".to_string(),
    };
    let findings = scan_doc_section(root, &opts, &[readme]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn doc_section_fails_when_heading_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let readme = root.join("agents/email/README.md");
    std::fs::create_dir_all(readme.parent().unwrap()).unwrap();
    std::fs::write(&readme, "# Email\n\nNo perf section.\n").unwrap();
    let opts = DocSectionOptions {
        glob: "agents/*/README.md".to_string(),
        required_heading: "## Performance".to_string(),
    };
    let findings = scan_doc_section(root, &opts, &[readme]).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing required heading"));
}

#[test]
fn doc_section_empty_opts_returns_empty() {
    let findings = scan_doc_section(Path::new("/tmp"), &DocSectionOptions::default(), &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn doc_section_skips_non_matching_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let other = root.join("other.md");
    std::fs::write(&other, "nothing here").unwrap();
    let opts = DocSectionOptions {
        glob: "agents/*/README.md".to_string(),
        required_heading: "## Performance".to_string(),
    };
    let findings = scan_doc_section(root, &opts, &[other]).unwrap();
    assert!(findings.is_empty());
}

// ── fixture-based integration tests ─────────────────────────────────────────

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture("rules/required-local-docs/pass");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
}

#[test]
fn fail_fixture_has_findings() {
    let root = fixture("rules/required-local-docs/fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let findings = check(&root, &config).unwrap();
    assert!(!findings.is_empty(), "expected findings but got none");
    assert!(findings.iter().all(|f| f.rule == RULE_ID));
}

#[test]
fn check_with_files_matches_check() {
    let root = fixture("rules/required-local-docs/fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let a = check(&root, &config).unwrap();
    let b = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(a, b);
}

#[test]
fn check_no_config_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = NoMistakesConfig::default();
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn findings_are_sorted() {
    let root = fixture("rules/required-local-docs/fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let findings = check(&root, &config).unwrap();
    for i in 1..findings.len() {
        assert!(findings[i - 1] <= findings[i]);
    }
}

#[test]
fn config_yaml_round_trip() {
    let config = config_with_yaml(
        RULE_ID,
        "roots: [agents]\nrequiredFile: README.md\ncodeExtensions: [mts, ts]\ntestExcludePatterns: [\"*.test.*\", __tests__]",
    );
    assert!(config.rule_configured(RULE_ID));
}

#[test]
fn doc_section_check_with_files_no_config_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = NoMistakesConfig::default();
    let findings = check_required_doc_section_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_required_doc_section_standalone_discovers_files() {
    // Exercises check_required_doc_section() (lines 18-25) which calls discover_files.
    // In a non-git temp dir, discover_files returns empty → no findings.
    let tmp = tempfile::tempdir().unwrap();
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: REQUIRED_DOC_SECTION_RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("glob: \"agents/*/README.md\"\nrequiredHeading: \"## Perf\"")
            .unwrap(),
        ..Default::default()
    });
    let findings = check_required_doc_section(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn doc_section_unreadable_file_is_skipped() {
    // Exercises Err(_) => continue (line 64): a file that matches the glob but
    // cannot be read should be silently skipped.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // ghost.md does NOT exist on disk, so read_to_string will fail
    let ghost = root.join("agents/email/README.md");
    std::fs::create_dir_all(ghost.parent().unwrap()).unwrap();
    // Don't write the file — it will fail to read
    let opts = DocSectionOptions {
        glob: "agents/*/README.md".to_string(),
        required_heading: "## Perf".to_string(),
    };
    let findings = scan_doc_section(root, &opts, &[ghost]).unwrap();
    // The file matches the glob but reading fails → no finding emitted
    assert!(
        findings.is_empty(),
        "unreadable file should be skipped silently"
    );
}

#[test]
fn scan_config_uses_defaults_when_fields_empty() {
    // Exercises the default-fallback branches of ScanConfig::new:
    // required_file empty → DEFAULT_REQUIRED_FILE
    // code_extensions empty → DEFAULT_CODE_EXTENSIONS
    // test_exclude_patterns empty → DEFAULT_TEST_EXCLUDE
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("agents/email");
    std::fs::create_dir_all(&dir).unwrap();
    let code_file = dir.join("index.ts"); // .ts is in DEFAULT_CODE_EXTENSIONS
    std::fs::write(&code_file, "").unwrap();
    let files = vec![code_file];
    // Options with empty required_file, code_extensions, test_exclude_patterns
    let opts = Options {
        roots: vec![std::path::PathBuf::from("agents")],
        required_file: String::new(), // → DEFAULT_REQUIRED_FILE (README.md)
        code_extensions: Vec::new(),  // → DEFAULT_CODE_EXTENSIONS
        test_exclude_patterns: Vec::new(), // → DEFAULT_TEST_EXCLUDE
    };
    let findings = scan(root, &opts, &files);
    // Should flag because README.md is missing
    assert_eq!(findings.len(), 1, "missing README.md should be flagged");
    assert!(findings[0].message.contains("README.md"));
}

#[test]
fn scan_with_absolute_root_path() {
    // Exercises root_rel.is_absolute() branch (line 138).
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("agents/email");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.ts"), "").unwrap();
    let files = vec![dir.join("index.ts")];
    // Use absolute path for root
    let abs_agents = root.join("agents");
    let opts = Options {
        roots: vec![abs_agents],
        required_file: "README.md".to_string(),
        code_extensions: vec!["ts".to_string()],
        test_exclude_patterns: vec!["*.test.*".to_string()],
    };
    let findings = scan(root, &opts, &files);
    assert_eq!(
        findings.len(),
        1,
        "missing README.md should be flagged with absolute root"
    );
}

#[test]
fn doc_section_findings_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for name in &["alpha", "beta", "gamma"] {
        let dir = root.join(format!("agents/{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "no heading").unwrap();
    }
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: REQUIRED_DOC_SECTION_RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("glob: \"agents/*/README.md\"\nrequiredHeading: \"## Perf\"")
            .unwrap(),
        ..Default::default()
    });
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    let findings = check_required_doc_section_with_files(root, &config, &files).unwrap();
    for i in 1..findings.len() {
        assert!(findings[i - 1] <= findings[i]);
    }
}
