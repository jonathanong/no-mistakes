use super::super::options::applications;
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/csp-allowed-origins/fixture")
            .join(name),
    )
}

fn deserialize_config(root: &Path, filename: &str) -> NoMistakesConfig {
    let source = std::fs::read_to_string(root.join(filename)).unwrap();
    serde_yaml::from_str(&source).unwrap()
}

#[test]
fn in_memory_applications_reject_invalid_options_and_scope() {
    let root = fixture("errors");
    for (config_file, expected) in [
        ("unknown-option.yml", "bogusOption"),
        ("empty-csp-files.yml", "requires non-empty options.cspFiles"),
        ("invalid-unanalyzable.yml", "options.unanalyzable"),
        ("invalid-include.yml", "unclosed character class"),
    ] {
        let config = deserialize_config(&root, config_file);
        let Err(error) = applications(&root, &config, &[]) else {
            panic!("{config_file}: expected an invalid application error");
        };
        assert!(
            format!("{error:#}").contains(expected),
            "{config_file}: {error:#}"
        );
    }
}

#[test]
fn applications_deduplicate_normalized_policy_paths() {
    let root = fixture("browser-semantics");
    let policy = root.join("src/duplicate.ts");
    let target = root.join("src/page.tsx");
    let config = deserialize_config(&root, "duplicate-paths.yml");
    let applications = applications(&root, &config, &[policy.clone(), target.clone()]).unwrap();

    assert_eq!(applications.len(), 1);
    assert_eq!(applications[0].csp_files, [policy]);
    assert_eq!(applications[0].target_files, [target]);
}
