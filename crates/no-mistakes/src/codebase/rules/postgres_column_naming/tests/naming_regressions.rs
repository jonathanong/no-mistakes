// cspell:ignore xuser xdelivery
use super::super::Options;
use super::support::messages;

fn fixture(scenario: &str) -> (String, serde_json::Value) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-column-naming/fixture")
        .join(scenario);
    let config: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let yaml = serde_yaml::to_string(&config["rules"][0]["options"]).unwrap();
    let catalog =
        serde_json::from_str(&std::fs::read_to_string(root.join("schema.json")).unwrap()).unwrap();
    (yaml, catalog)
}

#[test]
fn target_suffixes_accept_only_exact_bare_suffixes_and_suggest_without_repeating() {
    let (yaml, catalog) = fixture("bare-suffix-pass");
    assert!(messages(&yaml, catalog).is_empty());
    let (yaml, catalog) = fixture("bare-suffix-fail");
    let findings = messages(&yaml, catalog.clone());
    assert_eq!(
        findings,
        messages(
            &format!("{yaml}  allowReferencedColumnName: true\n"),
            catalog
        )
    );
    assert_eq!(findings.len(), 3);
    assert!(findings
        .iter()
        .any(|text| text.contains("column:orders.owner_id")));
    assert!(findings
        .iter()
        .any(|text| text.contains("column:audits.xuser_id")));
    assert!(findings
        .iter()
        .any(|text| text.contains("(for example user_id)")));
    assert!(findings
        .iter()
        .all(|text| !text.contains("(for example user_user_id)")));
}

#[test]
fn referenced_column_name_is_an_opt_in_with_word_boundaries() {
    let (yaml, catalog) = fixture("natural-key-pass");
    assert!(messages(&yaml, catalog.clone()).is_empty());
    assert!(messages(&yaml.replace("last-word", "full-name"), catalog).is_empty());
    // Default full-name matching still accepts keys that themselves name the target.
    let (yaml, catalog) = fixture("natural-key-default-pass");
    assert!(messages(&yaml, catalog).is_empty());
    let (yaml, catalog) = fixture("natural-key-default-fail");
    let findings = messages(&yaml, catalog.clone());
    assert_eq!(findings.len(), 4);
    let explicitly_off = format!("{yaml}  allowReferencedColumnName: false\n");
    assert_eq!(findings, messages(&explicitly_off, catalog));
    let (yaml, catalog) = fixture("natural-key-fail");
    let findings = messages(&yaml, catalog);
    assert_eq!(findings.len(), 3);
    for column in ["currency", "item_id", "xdelivery_key"] {
        assert!(findings.iter().any(|text| text.contains(column)));
    }
}

#[test]
fn referenced_column_option_rejects_unknown_keys_and_non_booleans() {
    for (yaml, expected) in [
        (
            "foreignKeys:\n  allowReferencedColumnNames: true\n",
            "unknown field",
        ),
        (
            "foreignKeys:\n  allowReferencedColumnName: sometimes\n",
            "boolean",
        ),
    ] {
        let error = serde_yaml::from_str::<Options>(yaml)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}
