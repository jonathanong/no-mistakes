use super::layout_check::has_extension;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

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

fn spec(
    root: &str,
    source_extension: &str,
    allowed_root_files: &[&str],
    allowed_subdirs: &[&str],
) -> PackageLayoutSpec {
    PackageLayoutSpec {
        root: PathBuf::from(root),
        source_extension: source_extension.to_string(),
        allowed_root_files: allowed_root_files.iter().map(|s| s.to_string()).collect(),
        allowed_subdirs: allowed_subdirs.iter().map(|s| s.to_string()).collect(),
    }
}

// ── check_relative unit tests ────────────────────────────────────────────────

fn globs() -> GlobSet {
    build_test_globset(&["*.test.mts", "*.spec.mts"])
}

#[test]
fn root_file_in_allowed_list_passes() {
    let s = spec("pkg", ".mts", &["package.json", "config.mts"], &[]);
    assert!(check_relative(
        Path::new("package.json"),
        &s,
        "__tests__",
        &globs(),
        "pkg/package.json"
    )
    .is_none());
}

#[test]
fn root_file_not_in_allowed_list_fails() {
    let s = spec("pkg", ".mts", &["package.json"], &[]);
    let msg = check_relative(
        Path::new("helpers.js"),
        &s,
        "__tests__",
        &globs(),
        "pkg/helpers.js",
    )
    .unwrap();
    assert!(msg.contains("root-level file must be"), "{msg}");
}

#[test]
fn root_test_file_passes() {
    let s = spec("pkg", ".mts", &[], &[]);
    assert!(check_relative(
        Path::new("index.test.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/index.test.mts"
    )
    .is_none());
}

#[test]
fn root_md_file_passes() {
    let s = spec("pkg", ".mts", &[], &[]);
    assert!(check_relative(
        Path::new("README.md"),
        &s,
        "__tests__",
        &globs(),
        "pkg/README.md"
    )
    .is_none());
}

#[test]
fn subdir_in_allowed_list_with_correct_ext_passes() {
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    assert!(check_relative(
        Path::new("enqueues/send.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/enqueues/send.mts"
    )
    .is_none());
}

#[test]
fn subdir_in_allowed_list_with_wrong_ext_fails() {
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    let msg = check_relative(
        Path::new("enqueues/send.js"),
        &s,
        "__tests__",
        &globs(),
        "pkg/enqueues/send.js",
    )
    .unwrap();
    assert!(msg.contains("must have extension .mts"), "{msg}");
}

#[test]
fn subdir_not_in_allowed_list_fails() {
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    let msg = check_relative(
        Path::new("utils/format.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/utils/format.mts",
    )
    .unwrap();
    assert!(msg.contains("subdirectory utils/ is not allowed"), "{msg}");
}

#[test]
fn test_dir_at_root_passes_with_test_file() {
    let s = spec("pkg", ".mts", &[], &[]);
    assert!(check_relative(
        Path::new("__tests__/a.test.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/__tests__/a.test.mts"
    )
    .is_none());
}

#[test]
fn test_dir_at_root_fails_non_test_file() {
    let s = spec("pkg", ".mts", &[], &[]);
    let msg = check_relative(
        Path::new("__tests__/helper.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/__tests__/helper.mts",
    )
    .unwrap();
    assert!(msg.contains("must match test file patterns"), "{msg}");
}

#[test]
fn allowed_subdir_with_test_dir_passes() {
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    assert!(check_relative(
        Path::new("enqueues/__tests__/a.test.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/enqueues/__tests__/a.test.mts"
    )
    .is_none());
}

#[test]
fn allowed_subdir_with_non_test_dir_fails() {
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    let msg = check_relative(
        Path::new("enqueues/deep/file.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/enqueues/deep/file.mts",
    )
    .unwrap();
    assert!(msg.contains("nested subdirectories"), "{msg}");
}

#[test]
fn deeply_nested_always_fails() {
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    let msg = check_relative(
        Path::new("enqueues/a/b/c.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/enqueues/a/b/c.mts",
    )
    .unwrap();
    assert!(msg.contains("nested subdirectories"), "{msg}");
}

#[test]
fn scan_derives_package_dirs_from_file_list_without_dir_metadata() {
    let root = Path::new("/workspace");
    let files = vec![root.join("packages/email/src/index.mts")];
    let opts = Options {
        test_file_patterns: Vec::new(),
        test_dir_name: String::new(),
        packages: vec![spec("packages", ".mts", &[], &[])],
    };
    let findings = scan(root, &opts, &files);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].file.contains("packages/email/src/index.mts"));
}

// Output cases moved to cli_rra_snapshot_strict_package_layout.

#[test]
fn check_with_files_returns_same_as_check() {
    let root = fixture("rules/strict-package-layout/fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let a = check(&root, &config).unwrap();
    let b = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(a, b);
}

#[test]
fn scan_returns_empty_for_no_packages() {
    let tmp = tempfile::tempdir().unwrap();
    let opts = Options::default();
    let result = scan(tmp.path(), &opts, &[]);
    assert!(result.is_empty());
}

#[test]
fn scan_uses_default_test_patterns_when_empty() {
    let root = fixture("rules/strict-package-layout/default-test-pattern");
    let file = root.join("pkg/index.test.ts");
    let opts = Options {
        test_file_patterns: vec![],
        test_dir_name: String::new(),
        packages: vec![PackageLayoutSpec {
            root: PathBuf::from("pkg"),
            source_extension: ".mts".to_string(),
            allowed_root_files: vec![],
            allowed_subdirs: vec![],
        }],
    };
    let findings = scan(&root, &opts, &[file]);
    assert!(
        findings.is_empty(),
        "*.test.ts should match default pattern *.test.*"
    );
}

#[test]
fn check_with_files_no_config_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = NoMistakesConfig::default();
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn findings_are_sorted() {
    let root = fixture("rules/strict-package-layout/fail");
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
        r#"
packages:
  - root: queues
    sourceExtension: .mts
    allowedRootFiles: [package.json]
    allowedSubdirs: [enqueues]
"#,
    );
    assert!(config.rule_configured(RULE_ID));
}

#[test]
fn scan_with_absolute_package_root() {
    // Exercises spec.root.is_absolute() branch (line 85).
    let root = fixture("rules/strict-package-layout/absolute-package-root");
    let file = root.join("queues/email/package.json");
    // Use absolute path for spec root
    let abs_queues = root.join("queues");
    let opts = Options {
        packages: vec![PackageLayoutSpec {
            root: abs_queues,
            source_extension: ".mts".to_string(),
            allowed_root_files: vec!["package.json".to_string()],
            allowed_subdirs: vec![],
        }],
        ..Options::default()
    };
    let findings = scan(&root, &opts, &[file]);
    // package.json is allowed at root level — no finding
    assert!(
        findings.is_empty(),
        "allowed root file should pass: {findings:#?}"
    );
}

fn normalized_package_root_selects_the_same_files(root_spec: &str) {
    let root = Path::new("/repo");
    let files = [
        "packages/api/package.json",
        "packages/api/index.ts",
        "packages/api/index.js",
        "packages/api/tmp/debug.ts",
        "other/note.ts",
    ]
    .into_iter()
    .map(|file| root.join(file))
    .collect::<Vec<_>>();
    let opts = Options {
        packages: vec![spec(
            root_spec,
            ".ts",
            &["package.json", "README.md", "index.ts"],
            &["src", "__tests__"],
        )],
        ..Options::default()
    };
    let files_flagged: Vec<_> = scan(root, &opts, &files)
        .into_iter()
        .map(|finding| finding.file)
        .collect();
    assert_eq!(
        files_flagged,
        vec![
            "packages/api/index.js".to_string(),
            "packages/api/tmp/debug.ts".to_string(),
        ]
    );
}

#[test]
fn dot_slash_and_parent_roots_match_packages() {
    normalized_package_root_selects_the_same_files("./packages");
    normalized_package_root_selects_the_same_files("packages/../packages");
    normalized_package_root_selects_the_same_files("packages");
}

#[test]
fn escaping_root_does_not_match_files_inside_or_outside_the_repo() {
    let root = Path::new("/repo");
    let files = vec![
        root.join("packages/api/tmp/debug.ts"),
        root.join("other/note.ts"),
    ];
    for root_spec in ["..", "../outside", "/"] {
        let opts = Options {
            packages: vec![spec(root_spec, ".ts", &["package.json"], &["src"])],
            ..Options::default()
        };
        assert!(
            scan(root, &opts, &files).is_empty(),
            "{root_spec} matched files"
        );
    }
}

#[test]
fn check_relative_empty_components_returns_none() {
    // Exercises the components.is_empty() guard (line 136) in check_relative.
    // rel = Path::new("") has no components.
    let s = spec("pkg", ".mts", &[], &[]);
    let globs = globs();
    let result = check_relative(Path::new(""), &s, "__tests__", &globs, "pkg/");
    assert!(result.is_none(), "empty rel path should return None");
}

#[test]
fn allowed_subdir_with_test_dir_non_test_file_fails() {
    // Exercises layout_check::check_two_deep (line 70-72): allowed subdir +
    // test dir but non-test file.
    let s = spec("pkg", ".mts", &[], &["enqueues"]);
    let msg = check_relative(
        Path::new("enqueues/__tests__/helper.mts"),
        &s,
        "__tests__",
        &globs(),
        "pkg/enqueues/__tests__/helper.mts",
    )
    .unwrap();
    assert!(msg.contains("must match test file patterns"), "{msg}");
}

#[test]
fn has_extension_exact_match() {
    assert!(has_extension("index.ts", "ts"));
    assert!(has_extension("index.ts", ".ts"));
    assert!(has_extension("index.mts", "mts"));
}

#[test]
fn has_extension_no_false_positive_on_suffix() {
    // "constants" and "assets" end with "ts" as a string but have no extension
    assert!(!has_extension("constants", "ts"));
    assert!(!has_extension("assets", "ts"));
}

#[test]
fn has_extension_wrong_extension() {
    assert!(!has_extension("index.js", "ts"));
}
