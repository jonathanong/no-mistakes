use super::*;
use serde_json::{json, Value};

fn bounded_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/bounded-import-closure"),
    )
    .display()
    .to_string()
}

#[test]
fn mixed_depth_same_bounds_keeps_unlimited_seed() {
    let root = bounded_root();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "shallow",
                    "files": ["web/app/page.tsx"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["web/**"],
                    "depth": 0,
                    "projection": "paths"
                },
                {
                    "type": "dependencies",
                    "id": "open",
                    "files": ["web/app/page.tsx"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["web/**"],
                    "projection": "paths"
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let reports = value["reports"].as_array().unwrap();
    let shallow = reports
        .iter()
        .find(|report| report["id"] == "shallow")
        .unwrap();
    let open = reports
        .iter()
        .find(|report| report["id"] == "open")
        .unwrap();
    let shallow_files = shallow["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    let open_files = open["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    assert!(
        !shallow_files
            .iter()
            .any(|path| path.contains("packages/ui")),
        "{shallow_files:?}"
    );
    assert!(
        open_files.iter().any(|path| path.contains("packages/ui")),
        "{open_files:?}"
    );
}

#[test]
fn reordered_candidate_globs_keep_exclusive_inventory() {
    let root = bounded_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [
                    {
                        "type": "dependencies",
                        "id": "first",
                        "files": ["web/app/page.tsx"],
                        "relationships": ["import-static", "import-dynamic", "import-type"],
                        "candidateInclude": ["web/**", "packages/**"],
                        "candidateExclude": ["**/*.test.*"],
                        "projection": "paths"
                    },
                    {
                        "type": "dependencies",
                        "id": "second",
                        "files": ["web/app/page.tsx"],
                        "relationships": ["import-static", "import-dynamic", "import-type"],
                        "candidateInclude": ["packages/**", "web/**"],
                        "candidateExclude": ["**/*.test.*"],
                        "projection": "paths"
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    assert!(
        !files.iter().any(|path| path.contains("page.test")),
        "{files:?}"
    );
    let work = observer.snapshot().work;
    assert!(work["graph.candidate_excluded"] >= 2, "{work:#?}");
}

#[test]
fn mixed_finite_depths_same_bounds_union_to_max() {
    let root = bounded_root();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "shallow",
                    "files": ["web/app/page.tsx"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["web/**"],
                    "depth": 0,
                    "projection": "paths"
                },
                {
                    "type": "dependencies",
                    "id": "deeper",
                    "files": ["web/app/page.tsx"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["web/**"],
                    "depth": 1,
                    "projection": "paths"
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let reports = value["reports"].as_array().unwrap();
    let shallow = reports
        .iter()
        .find(|report| report["id"] == "shallow")
        .unwrap();
    let deeper = reports
        .iter()
        .find(|report| report["id"] == "deeper")
        .unwrap();
    let shallow_files = shallow["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    let deeper_files = deeper["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    assert!(
        !shallow_files
            .iter()
            .any(|path| path.contains("packages/ui")),
        "{shallow_files:?}"
    );
    assert!(
        deeper_files.iter().any(|path| path.contains("packages/ui")),
        "{deeper_files:?}"
    );
}
