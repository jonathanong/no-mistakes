use super::{fixture_root, NoMistakesConfig};

#[test]
fn saved_bounded_rule_configs_parse_without_duplicate_executor_options() {
    for name in [
        "stale-allow.yml",
        "ignore-unanalyzable.yml",
        "suppress.yml",
        "valid.yml",
        "unanalyzable.yml",
        "embedded.yml",
        "delete-only.yml",
        "template.yml",
        "invalid.yml",
        "graph.yml",
        "keys.yml",
    ] {
        let yaml = std::fs::read_to_string(fixture_root().join(name)).unwrap();
        // Migration and original PR hygiene both selected this same module; keep one key.
        assert_eq!(yaml.matches("importSpecifier:").count(), 1, "{name}");
        let config: NoMistakesConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(
            config.rules[0].options["importSpecifier"].as_str(),
            Some("@example/db"),
            "{name}"
        );
    }
}
