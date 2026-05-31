use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

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

    assert!(scan_file(&root, &missing, &work).is_empty());
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

    assert!(scan_file(&root, &path, &work).is_empty());
}
