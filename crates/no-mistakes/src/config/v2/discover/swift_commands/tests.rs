use crate::config::v2::load_v2_config;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2")
        .join(name)
        .join("fixture")
}

#[test]
fn declared_package_argv_is_kept_literal() {
    let cfg = load_v2_config(&fixture("swift-base-commands"), None).unwrap();
    assert_eq!(
        cfg.tests
            .swift
            .base_commands
            .get("clients/android")
            .unwrap(),
        &vec!["host's swift".to_string(), "test".to_string()]
    );
    assert!(!cfg.tests.swift.base_commands.contains_key("clients/core"));
}

#[test]
fn unknown_package_names_the_declared_list() {
    let err = load_v2_config(&fixture("swift-base-commands-unknown"), None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("tests.swift.baseCommands.clients/missing"));
    assert!(err.contains("declared tests.swift.packages"));
}

#[test]
fn empty_argv_names_the_fix() {
    let err = load_v2_config(&fixture("swift-base-commands-empty"), None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("non-empty argv"));
    assert!(err.contains("[swift, test]"));
}

#[test]
fn blank_argument_rejects_shell_splitting() {
    let err = load_v2_config(&fixture("swift-base-commands-blank"), None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("baseCommands.clients/android[1]"));
    assert!(err.contains("literal argument"));
}

#[test]
fn trailing_slash_duplicate_is_one_package() {
    let err = load_v2_config(&fixture("swift-base-commands-duplicate"), None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("clients/android"));
    assert!(err.contains("more than once"));
}
