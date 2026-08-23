use super::queries::{
    call_sites_json_impl, dead_exports_json_impl, exports_of_json_impl, importers_json_impl,
    resolve_check_json_impl,
};

#[test]
fn importers_json_lists_direct_importers() {
    let options = json!({ "file": "util.ts", "root": fixture_root("queries") }).to_string();
    let output = importers_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["dependentsCount"], 3);
}

#[test]
fn exports_of_json_skips_importers_when_requested() {
    let options = json!({
        "file": "util.ts",
        "root": fixture_root("queries"),
        "noImporters": true,
    })
    .to_string();
    let output = exports_of_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["exports"][0]["importers"], json!([]));
}

#[test]
fn dead_exports_json_flags_dead() {
    let options = json!({ "file": "util.ts", "root": fixture_root("queries") }).to_string();
    let output = dead_exports_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["anyDead"], true);
}

#[test]
fn call_sites_json_reports_sites() {
    let options = json!({
        "file": "util.ts",
        "exportName": "used",
        "root": fixture_root("queries"),
    })
    .to_string();
    let output = call_sites_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["callSites"].as_array().unwrap().len(), 4);
}

#[test]
fn resolve_check_json_reports_unresolved() {
    let options = json!({ "file": "broken.ts", "root": fixture_root("queries") }).to_string();
    let output = resolve_check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["allResolve"], false);
}

#[test]
fn resolve_check_json_files_uses_batch_schema() {
    let options = json!({
        "files": ["consumer.ts"],
        "root": fixture_root("queries"),
    })
    .to_string();
    let output = resolve_check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["allResolve"], true);
    assert_eq!(value["results"][0]["file"], "consumer.ts");
    assert!(value.get("imports").is_none());
}

#[test]
fn resolve_check_json_files_rejects_empty_paths() {
    let error = resolve_check_json_impl(crate::napi_api::options::test_json_arg(json!({ "files": [""] }).to_string())).unwrap_err();
    assert!(error.reason.contains("files must not contain an empty path"));
}

#[test]
fn resolve_check_json_files_rejects_an_empty_list_with_an_agent_friendly_error() {
    let error = resolve_check_json_impl(crate::napi_api::options::test_json_arg(json!({ "files": [] }).to_string())).unwrap_err();
    assert_eq!(error.reason, "files must contain at least one path");
}

#[test]
fn pass4b_query_cli_and_napi_reports_share_gitignore_visibility() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let root_string = root.display().to_string();

    let cli_exports =
        crate::codebase::queries::exports_of::run_json(crate::codebase::queries::ExportsOfArgs {
            file: PathBuf::from("query/source.ts"),
            no_importers: true,
            root: Some(root.clone()),
            tsconfig: None,
            format: Some(crate::cli::Format::Json),
            json: true,
        })
        .unwrap();
    let napi_exports = exports_of_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "file": "query/source.ts",
            "noImporters": true,
            "root": root_string,
        })
        .to_string(),)
    )
    .unwrap();
    let cli_exports: serde_json::Value = serde_json::from_str(&cli_exports).unwrap();
    let napi_exports: serde_json::Value = serde_json::from_str(&napi_exports).unwrap();
    assert_eq!(napi_exports, cli_exports);
    assert!(napi_exports["exports"]
        .as_array()
        .unwrap()
        .iter()
        .all(|export| export["resolved"] == "query/target.ts"));

    let cli_resolve = crate::codebase::queries::resolve_check::run_json(
        crate::codebase::queries::ResolveCheckArgs {
            files: vec![PathBuf::from("query/source.ts")],
            root: Some(root),
            tsconfig: None,
            format: Some(crate::cli::Format::Json),
            json: true,
        },
    )
    .unwrap();
    let napi_resolve = resolve_check_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "file": "query/source.ts", "root": root_string }).to_string(),)
    )
    .unwrap();
    let cli_resolve: serde_json::Value = serde_json::from_str(&cli_resolve).unwrap();
    let napi_resolve: serde_json::Value = serde_json::from_str(&napi_resolve).unwrap();
    assert_eq!(napi_resolve, cli_resolve);
    assert_eq!(napi_resolve["allResolve"], true);
    assert!(napi_resolve["imports"]
        .as_array()
        .unwrap()
        .iter()
        .all(|import| import["resolved"] == "query/target.ts"));
}

#[test]
fn query_impls_require_inputs() {
    let missing_file = importers_json_impl(crate::napi_api::options::test_json_arg(json!({}).to_string())).unwrap_err();
    assert!(missing_file.reason.contains("file is required"));

    let missing_export =
        call_sites_json_impl(crate::napi_api::options::test_json_arg(json!({ "file": "util.ts" }).to_string())).unwrap_err();
    assert!(missing_export.reason.contains("exportName is required"));

    let resolve_ambiguous = resolve_check_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "file": "consumer.ts", "files": ["consumer.ts"] }).to_string(),)
    )
    .unwrap_err();
    assert!(resolve_ambiguous
        .reason
        .contains("exactly one of file or files is required"));
}
