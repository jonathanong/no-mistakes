use super::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn repo_fixture(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for part in parts {
        path.push(part);
    }
    crate::codebase::ts_resolver::normalize_path(&path)
}

fn parser_fixture(name: &str) -> PathBuf {
    repo_fixture(&["fixtures", "parser-count", name])
}

fn analysis_fixture(name: &str) -> PathBuf {
    repo_fixture(&["test-cases", "codebase-analysis", name, "fixture"])
}

fn category_fixture(category: &str, name: &str) -> PathBuf {
    repo_fixture(&["test-cases", category, name, "fixture"])
}

fn parse_json(output: String) -> Value {
    serde_json::from_str(&output).unwrap()
}

fn report_results(output: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(output).unwrap()["reports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|report| report["result"].clone())
        .collect()
}

fn assert_each_indexable_file_parsed_once(
    root: &Path,
    counts: &std::collections::HashMap<PathBuf, usize>,
) {
    let expected = crate::codebase::ts_source::discover_files(root, &[])
        .into_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .collect::<Vec<_>>();
    assert_eq!(counts.len(), expected.len(), "{counts:#?}");
    for path in expected {
        assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
    }
}

include!("domain_parity_tests/check_effects.rs");
include!("domain_parity_tests/repository_inventory.rs");
include!("domain_parity_tests/playwright_react_server.rs");
include!("domain_parity_tests/symbols.rs");
include!("domain_parity_tests/heterogeneous.rs");
