use super::*;
use std::collections::BTreeSet;

fn report(root: &std::path::Path, config: &std::path::Path) -> serde_json::Value {
    let json = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({"root": root, "config": config}).to_string(),
    ))
    .unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn component_demand_uses_existing_selection_without_poisoning_excluded_files() {
    let root = fixture("malformed-selected");
    let config = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/selected-advise.yml");
    let report = report(&root, &config);
    let failures: BTreeSet<_> = report["incomplete"]
        .as_array()
        .unwrap()
        .iter()
        .map(|failure| {
            assert_eq!(failure["consumer"], RULE_ID);
            let file = std::path::Path::new(failure["file"].as_str().unwrap());
            file.file_name().unwrap().to_str().unwrap().to_string()
        })
        .collect();
    assert_eq!(
        failures,
        BTreeSet::from([
            "Broken.tsx".into(),
            "FileDisabled.tsx".into(),
            "LineDisabled.tsx".into()
        ])
    );
}

#[test]
fn selected_source_failure_retains_healthy_component_but_failed_catalogs_block_absence() {
    let saved = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/storybook-partial");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let selected = report(root, &root.join(".no-mistakes.yml"));
    assert_eq!(
        selected["incomplete"].as_array().unwrap().len(),
        1,
        "{selected}"
    );
    assert!(selected["incomplete"][0]["file"]
        .as_str()
        .unwrap()
        .ends_with("Broken.tsx"));
    assert!(
        selected["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["file"] == "components/Good.tsx"),
        "{selected}"
    );
    for (config, failed) in [
        ("story.yml", "Broken.stories.tsx"),
        ("barrel.yml", "broken.ts"),
    ] {
        let result = report(root, &root.join(config));
        let failures = result["incomplete"].as_array().unwrap();
        assert_eq!(failures.len(), 1, "{config}: {result}");
        assert!(
            failures[0]["file"].as_str().unwrap().ends_with(failed),
            "{result}"
        );
        assert!(result["rules"].as_array().unwrap().is_empty(), "{result}");
        assert!(crate::codebase::rules::run_check(root, Some(&root.join(config)), None).is_err());
    }
    let valid = report(root, &root.join("valid.yml"));
    assert!(valid.get("incomplete").is_none(), "{valid}");
    assert!(valid["rules"].as_array().unwrap().is_empty(), "{valid}");
}

#[test]
fn unused_component_allowances_require_complete_selected_and_catalog_inputs() {
    let saved = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/storybook-partial");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let selected = report(root, &root.join("allowances.yml"));
    assert_eq!(
        selected["incomplete"].as_array().unwrap().len(),
        1,
        "{selected}"
    );
    let findings = selected["rules"].as_array().unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding["target"] == "components/Good.tsx#Missing"
                && finding["message"]
                    .as_str()
                    .unwrap()
                    .contains("does not match")
        }),
        "{selected}"
    );
    assert!(
        findings.iter().any(|finding| {
            finding["file"] == "components/Good.tsx"
                && finding["message"]
                    .as_str()
                    .unwrap()
                    .contains("no reachable story")
        }),
        "{selected}"
    );
    assert!(
        !findings
            .iter()
            .any(|finding| { finding["target"] == "components/Broken.tsx#Broken" }),
        "{selected}"
    );
    let catalog = report(root, &root.join("catalog-allowances.yml"));
    assert_eq!(
        catalog["incomplete"].as_array().unwrap().len(),
        1,
        "{catalog}"
    );
    assert!(catalog["rules"].as_array().unwrap().is_empty(), "{catalog}");
}
