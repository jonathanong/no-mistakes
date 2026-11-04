use super::validate;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-json-relation-ids")
        .join("config-keys")
        .join(name)
}

fn check(name: &str) -> anyhow::Result<()> {
    let path = fixture(name);
    let source = std::fs::read_to_string(&path).unwrap();
    validate(&source, &path)
}

#[test]
fn duplicate_decoded_json_keys_and_options_before_the_rule_are_detected() {
    for name in ["duplicate.json", "duplicate-reordered.yml"] {
        assert_eq!(
            check(name).unwrap_err().to_string(),
            "postgres-json-relation-ids option keyColumnMap: duplicate key accountId"
        );
    }
}

#[test]
fn other_rules_and_valid_config_shapes_are_left_to_the_config_parser() {
    for name in [
        "other-rule.jsonc",
        "other-rule.yml",
        "normal.jsonc",
        "empty.jsonc",
        "empty.yml",
    ] {
        check(name).unwrap();
    }
}

#[test]
fn malformed_and_unsupported_sources_keep_the_ordinary_parser_authoritative() {
    for name in [
        "no-rules.jsonc",
        "bad-syntax.jsonc",
        "bad-map.yml",
        "bad-syntax.yml",
        "bad-type.yml",
        "null.jsonc",
        "empty-source.jsonc",
        "unsupported.txt",
        "strict.json",
    ] {
        check(name).unwrap();
    }
}

#[test]
fn unrelated_option_shapes_do_not_mask_duplicates_in_the_real_loader() {
    for name in ["duplicate-mixed-options.yml", "duplicate-after-null.yml"] {
        let path = fixture(name);
        let root = path.parent().unwrap();
        let actual = crate::config::v2::load_v2_config(root, Some(&path))
            .unwrap_err()
            .to_string();
        assert_eq!(
            actual,
            "postgres-json-relation-ids option keyColumnMap: duplicate key accountId"
        );
    }
}

#[test]
fn fragment_duplicate_keeps_raw_validation_and_location() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/split-provenance/fixture/duplicate/.no-mistakes/rules/policy.yml");
    let source = std::fs::read_to_string(&path).unwrap();
    let error = super::validate_fragment(&source, &path, "rules")
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate key accountId"), "{error}");
    assert!(error.contains(&format!("{}:5:", path.display())), "{error}");
}

#[test]
fn fragment_duplicate_json_keeps_raw_validation_and_location() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/split-provenance/fixture/duplicate-json/.no-mistakes/rules/policy.json");
    let source = std::fs::read_to_string(&path).unwrap();
    let error = super::validate_fragment(&source, &path, "rules")
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate key accountId"), "{error}");
    assert!(error.contains(&format!("{}:5:", path.display())), "{error}");
}
