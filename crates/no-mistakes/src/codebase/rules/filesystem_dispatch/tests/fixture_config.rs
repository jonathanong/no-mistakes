use super::super::*;
use crate::codebase::postgres::{PREPARED_EMBEDDED_SQL_RULE_IDS, SCHEMA_CATALOG_RULE_IDS};

/// Copy the saved schema and option snippets. The rule list itself is rendered
/// from the registry so a newly merged filesystem rule stays on the config.
pub(super) fn all_rules_fixture() -> tempfile::TempDir {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/all-rules");
    crate::test_support::materialize_saved_fixture(&source)
}

pub(super) fn prepared_rules(rules: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let fixture = all_rules_fixture();
    let config_path = write_config(fixture.path(), rules);
    let loaded = crate::config::v2::load_v2_config(fixture.path(), Some(&config_path)).unwrap();
    let loaded_ids: Vec<_> = loaded.rules.iter().map(|rule| rule.rule.as_str()).collect();
    assert_eq!(
        loaded_ids, rules,
        "rendered dispatch config must enable the requested registry ids"
    );
    (fixture, config_path)
}

// Exceptional options live beside the rule's registry entry; shared SQL
// requirements continue to come from the corresponding SQL registries.
macro_rules! fixture_options {
    ($id:expr; $($rule:expr => $call:path => $options:expr),* $(,)?) => {
        [$(( $rule, $options ),)*]
            .into_iter()
            .find_map(|(rule, options)| (rule == $id).then_some(options))
            .unwrap_or("")
    };
}

fn options_block(id: &str) -> String {
    macro_rules! declared_options {
        ($($entry:tt)*) => { fixture_options!(id; $($entry)*) };
    }
    let declared = crate::filesystem_rules!(declared_options);
    let declared_keys: serde_yaml::Value = serde_yaml::from_str(declared).unwrap();
    let mut options = String::new();
    if SCHEMA_CATALOG_RULE_IDS.contains(&id) && declared_keys.get("schemaCatalogPath").is_none() {
        options.push_str("      schemaCatalogPath: schema.json\n");
    }
    if PREPARED_EMBEDDED_SQL_RULE_IDS.contains(&id) && declared_keys.get("executorNames").is_none()
    {
        options.push_str("      executorNames: []\n");
    }
    options.push_str(declared);
    options
}

fn write_config(dir: &std::path::Path, rules: &[&str]) -> std::path::PathBuf {
    let mut yaml = String::from("rules:\n");
    for id in rules {
        let options = options_block(id);
        yaml.push_str(&format!("  - rule: {id}\n    scope: repository\n"));
        if !options.is_empty() {
            yaml.push_str(&format!("    options:\n{options}"));
        }
    }
    let config_path = dir.join(".no-mistakes.yml");
    std::fs::write(&config_path, yaml).unwrap();
    config_path
}

#[test]
fn registry_declared_options_support_new_required_options() {
    // A newly registered rule supplies its options without a renderer ID branch.
    let options = fixture_options!("future-rule";
        "future-rule" => unused::check => "      requiredSetting: enabled\n",
    );
    let yaml: serde_yaml::Value = serde_yaml::from_str(options).unwrap();
    assert_eq!(yaml["requiredSetting"].as_str(), Some("enabled"));
    assert_eq!(
        fixture_options!("default-rule";
            "default-rule" => unused::check => "",
        ),
        ""
    );
}

#[test]
fn registry_options_keep_shared_sql_requirements_unique() {
    for id in FILESYSTEM_RULE_IDS {
        let options = options_block(id);
        let parsed: serde_yaml::Value = serde_yaml::from_str(&options).unwrap();
        if SCHEMA_CATALOG_RULE_IDS.contains(id) {
            assert_eq!(parsed["schemaCatalogPath"].as_str(), Some("schema.json"));
        }
        if PREPARED_EMBEDDED_SQL_RULE_IDS.contains(id) {
            assert_eq!(parsed["executorNames"].as_sequence().unwrap().len(), 0);
        }
    }
}
