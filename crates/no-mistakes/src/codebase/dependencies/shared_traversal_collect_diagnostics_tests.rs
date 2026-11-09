use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::codebase::ts_resolver::{TsConfigDiagnostic, TsConfigDiagnosticKind};

fn diagnostic(file: Option<&str>) -> TsConfigDiagnostic {
    TsConfigDiagnostic {
        kind: TsConfigDiagnosticKind::AmbiguousOwnership,
        config: None,
        file: file.map(PathBuf::from),
        detail: "two configs claim this source".to_string(),
        candidates: Vec::new(),
    }
}

#[test]
fn ordinary_diagnostics_exclude_portable_path_aliases_only() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/integration/napi-fixture-paths/fixture/paths.json"
    )))
    .unwrap();
    for case in cases["accepted"].as_array().unwrap() {
        let root = Path::new(case["root"].as_str().unwrap());
        let file = case["file"].as_str().unwrap();
        let expected = case["expected"].as_str().unwrap();
        let excluded = HashSet::from([root.join(expected)]);
        assert!(
            !super::ordinary_diagnostic_visible(root, &diagnostic(Some(file)), &excluded),
            "accepted path alias: {file}"
        );
        assert!(
            super::ordinary_diagnostic_visible(root, &diagnostic(Some(file)), &HashSet::new()),
            "empty exclusion set: {file}"
        );
        assert!(
            super::ordinary_diagnostic_visible(root, &diagnostic(None), &excluded),
            "diagnostic without a file: {file}"
        );
        assert!(
            super::ordinary_diagnostic_visible(
                root,
                &diagnostic(Some(file)),
                &HashSet::from([root.join("different.ts")])
            ),
            "different excluded file: {file}"
        );
    }
    for case in cases["rejected"].as_array().unwrap() {
        let root = Path::new(case["root"].as_str().unwrap());
        let file = case["file"].as_str().unwrap();
        let excluded = HashSet::from([root.join("setup.ts")]);
        assert!(
            super::ordinary_diagnostic_visible(root, &diagnostic(Some(file)), &excluded),
            "outside or unknown path: {file}"
        );
    }
    #[cfg(unix)]
    {
        let distinct = &cases["diagnosticDistinctPaths"];
        let root = Path::new(distinct["root"].as_str().unwrap());
        let file = distinct["file"].as_str().unwrap();
        let excluded = HashSet::from([root.join(distinct["excluded"].as_str().unwrap())]);
        assert!(
            super::ordinary_diagnostic_visible(root, &diagnostic(Some(file)), &excluded),
            "POSIX literal backslash must remain distinct from a path separator"
        );
    }
}
