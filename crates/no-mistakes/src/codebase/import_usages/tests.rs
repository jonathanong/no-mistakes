use super::model::package_name_from_specifier;
use super::*;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-usages/fixture")
}

fn args(files: Vec<&str>) -> ImportUsagesArgs {
    ImportUsagesArgs {
        files: files.into_iter().map(PathBuf::from).collect(),
        root: Some(fixture_root()),
        scan_roots: Vec::new(),
        filters: Vec::new(),
        format: Some(Format::Json),
        json: true,
        timings: false,
    }
}

#[test]
fn fact_collection_timeout_prevents_a_partial_report() {
    let _guard = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();

    let error = collect(&args(vec!["src/main.mts"])).unwrap_err();

    assert_eq!(crate::invocation::timeout_exit_code(&error), Some(124));
}

#[test]
fn package_name_handles_npm_boundaries() {
    assert_eq!(
        package_name_from_specifier("react/jsx-runtime").as_deref(),
        Some("react")
    );
    assert_eq!(
        package_name_from_specifier("@scope/pkg/register").as_deref(),
        Some("@scope/pkg")
    );
    assert_eq!(package_name_from_specifier("./local"), None);
    assert_eq!(package_name_from_specifier("#internal/register"), None);
    assert_eq!(package_name_from_specifier("node:fs/promises"), None);
    assert_eq!(package_name_from_specifier("https://esm.sh/react"), None);
    assert_eq!(
        package_name_from_specifier("data:text/javascript,export{}"),
        None
    );
    assert_eq!(package_name_from_specifier("@scope"), None);
}

#[test]
fn reports_all_direct_import_usage_rows() {
    let report = collect(&args(vec!["src/main.mts"])).unwrap();
    let file = report
        .files
        .iter()
        .find(|file| file.path == "src/main.mts")
        .unwrap();
    let rows: Vec<_> = file
        .imports
        .iter()
        .map(|row| {
            (
                row.specifier.as_str(),
                row.package_name.as_deref(),
                row.kind,
                row.line,
                row.side_effect_only,
                row.re_export,
            )
        })
        .collect();

    assert!(rows.contains(&("react", Some("react"), "static", 1, false, false)));
    assert!(rows.contains(&("zod", Some("zod"), "type", 2, false, false)));
    assert!(rows.contains(&("./setup.mts", None, "static", 3, true, false)));
    assert!(rows.contains(&(
        "@scope/pkg/helpers",
        Some("@scope/pkg"),
        "static",
        4,
        false,
        true,
    )));
    assert!(rows.contains(&(
        "remote-types",
        Some("remote-types"),
        "type",
        6,
        false,
        false
    )));
    assert!(rows.contains(&("next/dynamic", Some("next"), "dynamic", 9, false, false)));
    assert!(rows.contains(&("node:fs", None, "require", 10, false, false)));
    assert!(rows.contains(&(
        "@scope/pkg/register",
        Some("@scope/pkg"),
        "require-resolve",
        11,
        false,
        false,
    )));
    assert!(rows.contains(&("./local.cjs", None, "require", 12, false, false)));
}

#[test]
fn root_scan_and_filters_limit_source_files() {
    let mut scan_args = args(Vec::new());
    scan_args.filters = vec!["src/other.ts".to_string()];
    let report = collect(&scan_args).unwrap();

    assert_eq!(report.files.len(), 1);
    assert_eq!(report.files[0].path, "src/other.ts");
    assert_eq!(report.files[0].imports[0].specifier, "#internal/register");
    assert_eq!(report.files[0].imports[0].package_name, None);
    assert_eq!(
        report.files[0].imports[1].package_name.as_deref(),
        Some("lodash")
    );
}

#[test]
fn json_output_uses_camel_case_fields() {
    let json = run_json(args(vec!["src/main.mts"])).unwrap();
    assert!(json.contains("\"packageName\": \"react\""));
    assert!(json.contains("\"sideEffectOnly\": true"));
    assert!(json.contains("\"reExport\": true"));
    assert!(json.contains("\"kind\": \"require-resolve\""));
}
