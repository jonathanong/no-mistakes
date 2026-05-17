use super::*;
use std::path::PathBuf;

#[test]
fn rule_enabled_defaults_true_and_reads_false() {
    let config = Config::from_yaml(
        r#"
rules:
  disabled-rule:
    enabled: false
"#,
    )
    .unwrap();

    assert!(config.is_rule_enabled("missing-rule"));
    assert!(!config.is_rule_enabled("disabled-rule"));
}

#[test]
fn augment_from_gitignore_adds_plain_directory_names_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast-snippets/config/gitignore-project");
    let mut config = Config {
        filesystem: FilesystemConfig {
            skip_directories: vec!["dist".to_string()],
            skip_file_patterns: vec![],
        },
        rules: HashMap::new(),
    };

    config.augment_from_gitignore(&root);

    assert_eq!(
        config.filesystem.skip_directories,
        vec!["dist".to_string(), "node_modules".to_string()]
    );
}

#[test]
fn augment_from_gitignore_ignores_missing_file() {
    let mut config = Config::default();

    config.augment_from_gitignore(Path::new("/no/such/project"));

    assert!(config.filesystem.skip_directories.is_empty());
}
