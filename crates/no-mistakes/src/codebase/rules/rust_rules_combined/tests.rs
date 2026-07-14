use super::scan::{scan_file, scan_file_with_source};
use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

fn scan_test_file(root: &Path, path: &Path, work: &RustWork) -> Vec<RuleFinding> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    scan::scan_file(root, path, work, false, &sources)
}

fn config_with_rule(rule: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule.to_string(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn max_lines_work_uses_default_test_limit_for_test_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = root.join("tests/large.rs");
    let config = config_with_rule(RUST_MAX_LINES_PER_FILE);
    let mut work = BTreeMap::new();

    add_max_lines_work(&root, &config, std::slice::from_ref(&test_file), &mut work).unwrap();

    assert_eq!(
        work.get(&test_file).unwrap().max_limits,
        vec![rust_max_lines_per_file::DEFAULT_TEST_MAX]
    );
}

#[test]
fn scan_file_returns_empty_for_unreadable_file() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let missing = root.join("tests/does-not-exist.rs");
    let work = RustWork {
        max_limits: vec![1],
        inline_tests: true,
        inline_allows: true,
    };

    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&missing));
    assert!(scan_file(&root, &missing, &work, true, &sources).is_empty());
}

#[test]
fn scan_file_ignores_parse_errors_for_inline_rules() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/rust-combined/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let path = root.join("src/invalid.rs");
    let work = RustWork {
        max_limits: Vec::new(),
        inline_tests: true,
        inline_allows: true,
    };

    assert!(scan_test_file(&root, &path, &work).is_empty());
}

#[test]
fn combined_scan_applies_line_suppression_before_releasing_source() {
    let root = PathBuf::from("/repo");
    let path = root.join("src/lib.rs");
    let work = RustWork {
        inline_allows: true,
        ..Default::default()
    };
    let source = "// no-mistakes-disable-next-line rust-no-inline-allows\n#[allow(dead_code)]\nfn hidden() {}\n";

    assert!(scan_file_with_source(&root, &path, &work, source).is_empty());
}

#[test]
fn exclusive_sources_are_not_retained_and_overlapping_sources_are_memoized() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/rust-combined/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let path = root.join("src/lib.rs");
    let files = vec![path];

    let exclusive_sources = crate::codebase::rules::source_store_for_files(&files);
    let exclusive =
        check_with_files_and_sources(&root, &config, &files, &files, &exclusive_sources).unwrap();
    assert_eq!(exclusive_sources.physical_read_count(), 0);

    let overlapping_sources = crate::codebase::rules::source_store_for_files(&files);
    let overlapping =
        check_with_files_and_sources(&root, &config, &files, &[], &overlapping_sources).unwrap();
    assert_eq!(overlapping_sources.physical_read_count(), 1);
    assert_eq!(exclusive, overlapping);
}
