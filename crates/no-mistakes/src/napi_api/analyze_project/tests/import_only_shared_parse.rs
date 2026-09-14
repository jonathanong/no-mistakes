use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn lazy_import_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import/fixture"),
    )
}

fn report_paths(value: &Value, index: usize) -> Vec<String> {
    value["reports"][index]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["path"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn overlapping_import_only_dependency_reports_parse_each_file_once() {
    let root = lazy_import_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [
                    {
                        "id": "a",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "a-again",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "unrelated",
                        "type": "dependencies",
                        "files": ["src/unrelated.mts"],
                        "relationships": ["import-static"]
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let first = report_paths(&value, 0);
    let second = report_paths(&value, 1);
    let unrelated = report_paths(&value, 2);
    assert!(
        first.iter().any(|path| path.ends_with("src/b.mts")),
        "{first:?}"
    );
    assert_eq!(first, second);
    assert!(
        unrelated
            .iter()
            .any(|path| path.ends_with("src/unrelated-dep.mts")),
        "{unrelated:?}"
    );
    assert!(
        !first.iter().any(|path| path.contains("unrelated")),
        "{first:?}"
    );

    let work = observer.snapshot().work;
    assert_eq!(work["source.reads"], 4, "{work:#?}");
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(work["traversal.lazy_parallel_expand"], 1, "{work:#?}");
}

#[test]
fn overlapping_import_only_reports_share_one_graph_and_shared_neighbors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import-shared/fixture"),
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [
                    {
                        "id": "a",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "c",
                        "type": "dependencies",
                        "files": ["src/c.mts"],
                        "relationships": ["import-static"]
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let from_a = report_paths(&value, 0);
    let from_c = report_paths(&value, 1);
    assert!(
        from_a.iter().any(|path| path.ends_with("src/b.mts")),
        "{from_a:?}"
    );
    assert!(
        from_c.iter().any(|path| path.ends_with("src/b.mts")),
        "{from_c:?}"
    );
    assert!(
        !from_a.iter().any(|path| path.ends_with("src/c.mts")),
        "{from_a:?}"
    );
    assert!(
        !from_c.iter().any(|path| path.ends_with("src/a.mts")),
        "{from_c:?}"
    );

    let work = observer.snapshot().work;
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(work["traversal.lazy_parallel_expand"], 1, "{work:#?}");
}

#[test]
fn mixed_dependents_still_seed_later_import_only_reports() {
    let root = lazy_import_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [
                    {
                        "id": "dependents",
                        "type": "dependents",
                        "files": ["src/b.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "a",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "unrelated",
                        "type": "dependencies",
                        "files": ["src/unrelated.mts"],
                        "relationships": ["import-static"]
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let from_a = report_paths(&value, 1);
    let unrelated = report_paths(&value, 2);
    assert!(
        from_a.iter().any(|path| path.ends_with("src/b.mts")),
        "{from_a:?}"
    );
    assert!(
        unrelated
            .iter()
            .any(|path| path.ends_with("src/unrelated-dep.mts")),
        "{unrelated:?}"
    );
    assert!(
        !from_a.iter().any(|path| path.contains("unrelated")),
        "{from_a:?}"
    );
    let work = observer.snapshot().work;
    assert!(work["graph.builds"] >= 1, "{work:#?}");
}

#[test]
fn ineligible_reports_do_not_block_import_only_graph_seed() {
    let root = lazy_import_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [
                    {
                        "id": "missing",
                        "type": "dependencies",
                        "files": ["src/missing.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "symbols",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"],
                        "includeSymbols": true
                    },
                    {
                        "id": "workspace",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["workspace"]
                    },
                    {
                        "id": "a",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "unrelated",
                        "type": "dependencies",
                        "files": ["src/unrelated.mts"],
                        "relationships": ["import-static"]
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let from_a = report_paths(&value, 3);
    let unrelated = report_paths(&value, 4);
    assert!(
        from_a.iter().any(|path| path.ends_with("src/b.mts")),
        "{from_a:?}"
    );
    assert!(
        unrelated
            .iter()
            .any(|path| path.ends_with("src/unrelated-dep.mts")),
        "{unrelated:?}"
    );
    let work = observer.snapshot().work;
    assert!(work["graph.builds"] >= 1, "{work:#?}");
}

#[test]
fn import_only_union_merges_distinct_relationships() {
    let root = lazy_import_root();
    let value: Value = serde_json::from_str(
        &analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [
                    {
                        "id": "static",
                        "type": "dependencies",
                        "files": ["src/a.mts"],
                        "relationships": ["import-static"]
                    },
                    {
                        "id": "dynamic",
                        "type": "dependencies",
                        "files": ["src/unrelated.mts"],
                        "relationships": ["import-dynamic"]
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let from_a = report_paths(&value, 0);
    assert!(
        from_a.iter().any(|path| path.ends_with("src/b.mts")),
        "{from_a:?}"
    );
    assert_eq!(value["reports"].as_array().unwrap().len(), 2);
}
