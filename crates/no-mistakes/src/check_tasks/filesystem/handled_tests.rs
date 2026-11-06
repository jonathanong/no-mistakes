use super::*;
use std::path::Path;

#[test]
fn handled_applications_keep_authored_slots_and_skip_future_dispatch() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/unused-allow-string-locations");
    let mut config = no_mistakes::config::v2::load_v2_config(&root, None).unwrap();
    let original = config.rules.clone();
    mark(&mut config, |id| id == "package-json-required-fields");
    assert!(!config.rule_configured("package-json-required-fields"));
    assert_eq!(config.rules.len(), original.len());
    for (before, after) in original.iter().zip(&config.rules) {
        assert_eq!(before.name, after.name);
        assert_eq!(before.rule, after.rule);
        assert_eq!(before.options, after.options);
    }
    for (index, id) in [
        (25, "lockfile-allowlist"),
        (26, "file-extension-policy"),
        (27, "package-json-workspace-coverage"),
    ] {
        assert_eq!(config.rules[index], original[index]);
        assert!(config.rule_configured(id));
    }
}
