use super::ConfigAnchor;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

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
