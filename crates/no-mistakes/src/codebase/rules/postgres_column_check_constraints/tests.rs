use super::*;
use crate::config::v2::load_v2_config;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-column-check-constraints")
}

#[test]
fn standalone_uses_the_same_saved_catalog_and_options() {
    let root = fixture().join("fixture/fail");
    for (name, count) in [
        ("catalog", 28),
        ("allow", 27),
        ("no-catalog", 0),
        ("empty", 0),
        ("excluded", 0),
    ] {
        let config =
            load_v2_config(&root, Some(&fixture().join(format!("configs/{name}.yml")))).unwrap();
        let files = vec![root.join("schema.json")];
        assert_eq!(
            check_with_files(&root, &config, &files).unwrap().len(),
            count,
            "{name}"
        );
    }
}

#[test]
fn standalone_validates_options_before_reading_the_catalog() {
    let root = fixture().join("fixture/missing-catalog");
    for (name, expected) in [
        ("bad-type", "unsupported integer type"),
        ("bad-predicate", "non-negative"),
        ("bad-reason", "needs a reason"),
        ("invalid-allow", "table.column"),
        ("bad-mode", "unknown field"),
    ] {
        let config =
            load_v2_config(&root, Some(&fixture().join(format!("configs/{name}.yml")))).unwrap();
        let error = check_with_files(&root, &config, &[]).unwrap_err();
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
}

#[test]
fn typed_options_round_trip_the_saved_public_contract() {
    let root = fixture().join("fixture/fail");
    let config = load_v2_config(&root, Some(&fixture().join("configs/allow.yml"))).unwrap();
    let options: Options = config.rule_applications(RULE_ID)[0]
        .try_rule_options()
        .unwrap();
    let json = serde_json::to_value(&options).unwrap();
    assert_eq!(json["schemaCatalogPath"], "schema.json");
    assert_eq!(json["columns"][0]["require"], "non-negative");
    assert_eq!(json["allow"][0]["column"], "metrics.missing_count");
    assert_eq!(json["allow"][0]["reason"], "signed delta by design");
    let restored: Options =
        serde_yaml::from_str(&serde_yaml::to_string(&options).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(restored).unwrap(), json);
}
