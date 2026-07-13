use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

include!("options_config_tests.rs");

#[test]
fn report_options_merge_top_level_defaults() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "tsconfig": "tsconfig.json",
            "config": "no-mistakes.json",
            "filters": ["src/**"],
            "reports": [
                { "type": "symbols", "root": "override", "files": ["a.mts"] },
                { "type": "playwrightCheck" },
                { "type": "queues" },
                { "type": "dependencies", "files": ["a.mts"] }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["root"], "override");
    assert_eq!(
        symbols["tsconfig"],
        format!("{}/tsconfig.json", fixture_root("simple"))
    );
    assert!(symbols.get("filters").is_none());
    assert_eq!(
        symbols["config"],
        format!("{}/no-mistakes.json", fixture_root("simple"))
    );

    let playwright: Value =
        serde_json::from_str(&options::playwright_options(&options.reports[1], &options).unwrap())
            .unwrap();
    assert_eq!(playwright["root"], fixture_root("simple"));
    assert_eq!(
        playwright["config"],
        format!("{}/no-mistakes.json", fixture_root("simple"))
    );
    assert!(playwright.get("tsconfig").is_none());
    assert!(playwright.get("filters").is_none());

    let project: Value =
        serde_json::from_str(&options::project_options(&options.reports[2], &options).unwrap())
            .unwrap();
    assert_eq!(
        project["config"],
        format!("{}/no-mistakes.json", fixture_root("simple"))
    );

    let traverse = options::traverse_options(&options.reports[3], &options).unwrap();
    assert_eq!(traverse.filters, vec!["src/**"]);

    let inherited_symbols = serde_json::from_str::<Value>(
        &options::symbols_options(&options.reports[3], &options).unwrap(),
    )
    .unwrap();
    assert_eq!(inherited_symbols["root"], fixture_root("simple"));
}

#[test]
fn report_options_forward_relative_top_level_tsconfig_from_root() {
    let root = fixture_root("forbidden-dependencies-passes");
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": root,
            "tsconfig": "tsconfig.json",
            "reports": [{ "type": "symbols", "files": ["src/index.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["tsconfig"], format!("{root}/tsconfig.json"));
}

#[test]
fn report_options_keep_per_report_tsconfig_override() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "tsconfig": "tsconfig.json",
            "reports": [{
                "type": "symbols",
                "files": ["a.mts"],
                "tsconfig": "custom.json"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["tsconfig"], "custom.json");
}

#[test]
fn report_options_forward_absolute_top_level_tsconfig() {
    let absolute = PathBuf::from(fixture_root("forbidden-dependencies-passes"))
        .join("tsconfig.json")
        .display()
        .to_string();
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "tsconfig": absolute,
            "reports": [{ "type": "symbols", "files": ["a.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["tsconfig"], absolute);
}

#[test]
fn report_options_keep_relative_top_level_tsconfig_without_root() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "tsconfig": "tsconfig.json",
            "reports": [{ "type": "symbols", "files": ["a.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["tsconfig"], "tsconfig.json");
}

#[test]
fn report_options_allow_missing_top_level_root() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "reports": [{ "type": "symbols", "files": ["a.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert!(symbols.get("root").is_none());
}

#[test]
fn resolve_helpers_handle_defaults_and_explicit_paths() {
    let cwd = std::env::current_dir().unwrap();
    let default_root = options::resolve_root(None).unwrap();
    assert_eq!(
        default_root,
        crate::codebase::ts_resolver::normalize_path(&cwd)
    );

    let root = PathBuf::from(fixture_root("simple"));
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let fallback = options_test_support::resolve_tsconfig(&root, None, &visible).unwrap();
    assert_eq!(fallback.paths_dir, root);

    let ts_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/forbidden-dependencies-passes/fixture");
    let visible = crate::codebase::ts_source::discover_visible_paths(&ts_root);
    let explicit =
        options_test_support::resolve_tsconfig(&ts_root, Some("tsconfig.json"), &visible).unwrap();
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&explicit.dir),
        crate::codebase::ts_resolver::normalize_path(&ts_root)
    );

    let found = options_test_support::resolve_tsconfig(&ts_root, None, &visible).unwrap();
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&found.dir),
        crate::codebase::ts_resolver::normalize_path(&ts_root)
    );

    let absolute_path = ts_root.join("tsconfig.json");
    let absolute =
        options_test_support::resolve_tsconfig(&ts_root, absolute_path.to_str(), &visible).unwrap();
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&absolute.dir),
        crate::codebase::ts_resolver::normalize_path(&ts_root)
    );
}
