use super::options;
use super::options_test_support::parse_options;
use super::types::AnalyzeProjectOptions;
use serde_json::json;

fn fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

#[test]
fn report_option_helpers_cover_success_shapes_and_relative_roots() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": "crates/no-mistakes",
            "tsconfig": "tsconfig.json",
            "config": "no-mistakes.json",
            "reports": [
                { "type": "importUsages" },
                { "type": "flow" },
                { "type": "effects", "kind": "fetch" },
                { "type": "rsc-callers", "component": "Home" },
                { "type": "dependencies", "files": ["a.mts"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    assert!(options::import_usages_options(&options.reports[0], &options).is_ok());
    assert!(options::flow_options(&options.reports[1], &options).is_ok());
    assert!(options::effects_options(&options.reports[2], &options).is_ok());
    assert!(options::rsc_callers_options(&options.reports[3], &options).is_ok());
    let traverse = options::traverse_options(&options.reports[4], &options).unwrap();
    assert!(traverse.filters.is_empty());

    let relative = options::resolve_root(Some("crates/no-mistakes")).unwrap();
    assert!(relative.ends_with("crates/no-mistakes"));
}

#[test]
fn command_options_cover_remaining_merge_flag_groups() {
    let root = fixture_root("simple");
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": root,
            "tsconfig": "tsconfig.json",
            "config": "no-mistakes.json",
            "reports": [
                { "type": "exportsOf", "file": "a.mts" },
                { "type": "deadExports" },
                { "type": "callSites", "file": "a.mts" },
                { "type": "resolveCheck", "file": "a.mts" },
                { "type": "fetches" },
                { "type": "dataPw" },
                { "type": "ciEnv" },
                { "type": "ciTopology" },
                { "type": "infraResourceRefs" },
                { "type": "infraOutputs" },
                { "type": "infraTestFor" },
                { "type": "swiftImporters" },
                { "type": "swiftTestTargets" },
                { "type": "testsImpact" },
                { "type": "testsTargets" },
                { "type": "testsWhy" },
                { "type": "impactedChecks" },
                { "type": "unknownReport" }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let exporters = options::command_options(&options.reports[0], &options).unwrap();
    assert_eq!(exporters["tsconfig"], format!("{root}/tsconfig.json"));
    assert!(exporters.get("config").is_none());

    let fetches = options::command_options(&options.reports[4], &options).unwrap();
    assert_eq!(fetches["config"], format!("{root}/no-mistakes.json"));
    assert!(fetches.get("tsconfig").is_none());

    let tests_impact = options::command_options(&options.reports[13], &options).unwrap();
    assert_eq!(tests_impact["tsconfig"], format!("{root}/tsconfig.json"));
    assert_eq!(tests_impact["config"], format!("{root}/no-mistakes.json"));

    let unknown = options::command_options(&options.reports[17], &options).unwrap();
    assert!(unknown.get("root").is_none());
    assert!(unknown.get("tsconfig").is_none());
    assert!(unknown.get("config").is_none());
}
