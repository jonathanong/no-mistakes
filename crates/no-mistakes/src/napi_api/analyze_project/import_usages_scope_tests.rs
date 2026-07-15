use super::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/napi/analyze-project-import-usages-scope"),
    )
}

fn parse_json(value: String) -> Value {
    serde_json::from_str(&value).unwrap()
}

fn aggregate_import_usages(
    root: &Path,
    report: Value,
) -> (
    Value,
    std::sync::Arc<crate::diagnostics::InvocationObserver>,
) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [report]
            })
            .to_string(),
        )
        .unwrap()
    };
    (parse_json(output)["reports"][0]["result"].clone(), observer)
}

fn assert_single_reads(observer: &crate::diagnostics::InvocationObserver, expected: &[PathBuf]) {
    let expected = expected
        .iter()
        .cloned()
        .map(|path| (path, 1))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(observer.source_read_snapshot(), expected);
}

#[test]
fn external_scan_root_matches_standalone_and_is_prepared_once() {
    let fixture = fixture_root();
    let root = fixture.join("report-root");
    let external = fixture.join("external-root");
    let standalone = parse_json(
        crate::napi_api::import_usages_json_impl(
            json!({ "root": root, "scanRoots": [external] }).to_string(),
        )
        .unwrap(),
    );
    let (aggregate, observer) = aggregate_import_usages(
        &root,
        json!({ "type": "importUsages", "scanRoots": [external] }),
    );

    assert_eq!(aggregate, standalone);
    assert_eq!(aggregate["files"].as_array().unwrap().len(), 1);
    assert_eq!(
        aggregate["files"][0]["path"],
        external.join("external.ts").display().to_string()
    );
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 2, "{work:#?}");
    assert_eq!(work["discovery.requests"], 2, "{work:#?}");
    assert_eq!(work["discovery.cache_hits"], 1, "{work:#?}");
    assert_eq!(work["source.reads"], 3, "{work:#?}");
    assert_eq!(work["parse.files"], 3, "{work:#?}");
    assert_single_reads(
        &observer,
        &[
            root.join("src/entry.ts"),
            root.join("src/helper.ts"),
            external.join("external.ts"),
        ],
    );
}

#[test]
fn omitted_files_reuse_the_same_root_snapshot_without_a_second_discovery() {
    let root = fixture_root().join("report-root");
    let standalone = parse_json(
        crate::napi_api::import_usages_json_impl(json!({ "root": root }).to_string()).unwrap(),
    );
    let (aggregate, observer) = aggregate_import_usages(&root, json!({ "type": "importUsages" }));

    assert_eq!(aggregate, standalone);
    assert_eq!(aggregate["files"].as_array().unwrap().len(), 2);
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["discovery.requests"], 1, "{work:#?}");
    assert_eq!(work["discovery.cache_hits"], 1, "{work:#?}");
    assert_eq!(work["source.reads"], 2, "{work:#?}");
    assert_eq!(work["parse.files"], 2, "{work:#?}");
    assert_single_reads(
        &observer,
        &[root.join("src/entry.ts"), root.join("src/helper.ts")],
    );
}

#[test]
fn equivalent_reports_share_one_prepared_file_universe() {
    let root = fixture_root().join("report-root");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        parse_json(
            analyze_project_json_impl(
                json!({
                    "root": root,
                    "reports": [
                        { "type": "importUsages", "id": "first" },
                        { "type": "importUsages", "id": "second" }
                    ]
                })
                .to_string(),
            )
            .unwrap(),
        )
    };

    assert_eq!(
        output["reports"][0]["result"],
        output["reports"][1]["result"]
    );
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["discovery.requests"], 1, "{work:#?}");
    assert_eq!(work["source.reads"], 2, "{work:#?}");
    assert_eq!(work["parse.files"], 2, "{work:#?}");
}

include!("import_usages_scope_tests/cross_scope.rs");
