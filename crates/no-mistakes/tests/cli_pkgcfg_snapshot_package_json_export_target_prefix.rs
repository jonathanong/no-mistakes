use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "package-json-export-target-prefix";

#[test]
fn package_json_export_target_prefix_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let cases = ["default", "advisory", "disabled", "invalid-options"].map(|case| {
        (
            case,
            root.clone(),
            rules_path(RULE, "configs", &format!("{case}.yml")),
        )
    });
    assert_saved_fixture("package_json_export_target_prefix", &cases);
}

#[test]
fn standalone_package_json_export_target_prefix_reports_authored_key() {
    use no_mistakes::codebase::rules::package_json_export_target_prefix::check;
    use no_mistakes::config::v2::discover::load_v2_config;

    let root = rules_path(RULE, "fixture", "");
    let config_path = rules_path(RULE, "configs", "demo.yml");
    let config = load_v2_config(&root, Some(&config_path)).expect("load saved rule configuration");
    let findings = check(&root, &config).expect("check package manifest");
    let locations: Vec<_> = findings
        .iter()
        .map(|finding| {
            (
                finding.file.as_str(),
                finding.line,
                finding.target.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        locations,
        [(
            "packages/prepare/package.json",
            5,
            Some("/exports/./import")
        )],
    );
}
