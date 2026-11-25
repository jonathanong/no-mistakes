use super::*;
use serde_json::{json, Value};

#[test]
fn check_json_reports_unconditional_swift_darwin_imports() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/swift-no-unguarded-darwin-import");
    let root = fixture.join("fixture");
    let config = fixture.join("configs/check.yml");
    let result: Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "config": config, "includeSuppressed": true }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let rules = result["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 15, "{result}");
    assert_eq!(
        rules
            .iter()
            .filter(|finding| finding["target"] == "unanalyzable")
            .count(),
        6
    );
    assert_eq!(rules[0]["file"], "Tests/ClockTests.swift");
    assert_eq!(rules[0]["line"], 2);
    assert_eq!(rules[0]["rule"], "swift-no-unguarded-darwin-import");
    let normalized_root = crate::codebase::ts_resolver::normalize_path(&root);
    let parsed_config = crate::config::v2::load_v2_config(&normalized_root, Some(&config)).unwrap();
    let files = crate::codebase::ts_source::discover_files(&normalized_root, &[]);
    let library = crate::codebase::rules::swift_no_unguarded_darwin_import::check_with_files(
        &normalized_root,
        &parsed_config,
        &files,
    )
    .unwrap();
    assert_eq!(serde_json::to_value(library).unwrap(), result["rules"]);

    let suppressed = result["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 4, "{result}");

    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({
                    "root": root,
                    "config": config,
                    "reports": [{ "type": "check", "includeSuppressed": true }]
                })
                .to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"][0]["result"], result);
}

#[test]
fn check_json_uses_configured_swift_import_modules() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/swift-no-unguarded-darwin-import");
    let result: Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": fixture.join("apple-modules"),
                "config": fixture.join("configs/modules.yml"),
            })
            .to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let lines: Vec<_> = result["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [2, 7, 27, 49]);
}
