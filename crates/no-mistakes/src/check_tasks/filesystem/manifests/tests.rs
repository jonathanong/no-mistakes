use super::*;
use no_mistakes::codebase::ts_source::{FileInventory, SourceStore};
use no_mistakes::config::v2::schema::RuleDef;
use std::sync::Arc;

#[test]
fn unrelated_rules_borrow_the_original_config_without_source_work() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: rules::BANNED_PATHS.to_owned(),
        ..Default::default()
    });
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    let (remaining, diagnostics) = run(Path::new("."), &config, &[], &sources, false).unwrap();
    assert!(matches!(remaining, Cow::Borrowed(original) if std::ptr::eq(original, &config)));
    assert!(diagnostics.findings.is_empty());
    assert!(diagnostics.advisories.is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}
