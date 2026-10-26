use super::ConfigAnchor;
use crate::config::v2::schema::loaded_path::LoadedConfigPath;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

#[test]
fn a_config_without_a_loaded_path_stays_on_the_conventional_line() {
    let config = NoMistakesConfig::default();
    let rule = RuleDef::default();
    let anchor = ConfigAnchor::resolve(Path::new("/repo"), &config, None, &rule);
    let finding = anchor.emit(
        "production-dependency-declarations",
        anchor.option_line("workspaceRoots"),
        "message".to_string(),
        None,
        None,
    );
    assert_eq!(finding.file, ".no-mistakes.yml");
    assert_eq!(finding.line, 1);
    assert_eq!(anchor.option_item_line("extraGeneratedColumns", 0), 1);
}

#[test]
fn a_loaded_path_without_a_source_store_keeps_line_one() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/config-diagnostic-locations/fixture/production-custom");
    let config = NoMistakesConfig {
        loaded_config_path: LoadedConfigPath(Some(root.join("check.yml"))),
        rules: vec![RuleDef {
            rule: "production-dependency-declarations".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let anchor = ConfigAnchor::resolve(&root, &config, None, &config.rules[0]);
    let finding = anchor.emit(
        "production-dependency-declarations",
        anchor.option_line("workspaceRoots"),
        "message".to_string(),
        None,
        None,
    );
    assert_eq!(finding.file, "check.yml");
    assert_eq!(finding.line, 1);
}
