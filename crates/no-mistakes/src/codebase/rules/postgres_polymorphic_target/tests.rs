use super::*;
use crate::config::v2::load_v2_config;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-polymorphic-target")
}

#[test]
fn standalone_uses_the_same_saved_catalog_and_options() {
    let root = fixture().join("fixture/fail");
    for (name, count) in [
        ("catalog", 6),
        ("allow", 4),
        ("custom-suffix", 1),
        ("excluded", 0),
        ("off", 0),
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
        ("missing-path", "schemaCatalogPath: required"),
        ("empty-type-suffix", "typeSuffix: must not be empty"),
        ("empty-id-suffix", "idSuffix: must not be empty"),
        ("same-suffix", "typeSuffix and idSuffix must be different"),
        ("bad-reason", "needs a reason"),
        ("invalid-allow", "table.column"),
        ("duplicate-allow", "duplicate entry"),
        ("unknown-option", "unknown field"),
    ] {
        let path = fixture().join(format!("configs/{name}.yml"));
        let config = serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
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
    assert_eq!(json["typeSuffix"], "_type");
    assert_eq!(json["idSuffix"], "_id");
    assert_eq!(json["allow"][0]["column"], "audit_entries.subject_id");
    assert_eq!(
        json["allow"][0]["reason"],
        "immutable audit history preserves deleted subjects"
    );
    let restored: Options =
        serde_yaml::from_str(&serde_yaml::to_string(&options).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(restored).unwrap(), json);
}
