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
    assert_eq!(symbols["tsconfig"], "tsconfig.json");
    assert!(symbols.get("filters").is_none());
    assert!(symbols.get("config").is_none());

    let playwright: Value =
        serde_json::from_str(&options::playwright_options(&options.reports[1], &options).unwrap())
            .unwrap();
    assert_eq!(playwright["root"], fixture_root("simple"));
    assert_eq!(playwright["config"], "no-mistakes.json");
    assert!(playwright.get("tsconfig").is_none());
    assert!(playwright.get("filters").is_none());

    let project: Value =
        serde_json::from_str(&options::project_options(&options.reports[2], &options).unwrap())
            .unwrap();
    assert_eq!(project["config"], "no-mistakes.json");

    let traverse = options::traverse_options(&options.reports[3], &options).unwrap();
    assert_eq!(traverse.filters, vec!["src/**"]);

    let inherited_symbols = serde_json::from_str::<Value>(
        &options::symbols_options(&options.reports[3], &options).unwrap(),
    )
    .unwrap();
    assert_eq!(inherited_symbols["root"], fixture_root("simple"));
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
    let fallback = options::resolve_tsconfig(&root, None).unwrap();
    assert_eq!(fallback.paths_dir, root);

    let ts_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/forbidden-dependencies-passes/fixture");
    let explicit = options::resolve_tsconfig(&ts_root, Some("tsconfig.json")).unwrap();
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&explicit.dir),
        crate::codebase::ts_resolver::normalize_path(&ts_root)
    );

    let found = options::resolve_tsconfig(&ts_root, None).unwrap();
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&found.dir),
        crate::codebase::ts_resolver::normalize_path(&ts_root)
    );

    let absolute_path = ts_root.join("tsconfig.json");
    let absolute = options::resolve_tsconfig(&ts_root, absolute_path.to_str()).unwrap();
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&absolute.dir),
        crate::codebase::ts_resolver::normalize_path(&ts_root)
    );
}
