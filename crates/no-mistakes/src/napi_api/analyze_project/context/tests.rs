use super::{
    cached_analysis, canonical_filter_key, framework_preparation_plan, graph_build_plan,
    same_config_path, CachedAnalysis,
};
use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn aggregate_graph_reports_union_explicit_framework_demand() {
    let options: crate::napi_api::analyze_project::types::AnalyzeProjectOptions =
        serde_json::from_value(serde_json::json!({
            "reports": [
                {
                    "type": "dependencies",
                    "files": ["src/a.ts"],
                    "relationships": ["import"],
                    "tests": ["vitest"]
                },
                {
                    "type": "dependents",
                    "files": ["src/b.ts"],
                    "relationships": ["import"],
                    "tests": ["swift"]
                }
            ]
        }))
        .unwrap();
    let graph = graph_build_plan(&options).unwrap();
    let frameworks = framework_preparation_plan(&options, graph).unwrap();

    assert!(frameworks.contains(crate::codebase::test_discovery::TestRunner::Vitest));
    assert!(frameworks.contains(crate::codebase::test_discovery::TestRunner::Swift));
    assert!(!frameworks.contains(crate::codebase::test_discovery::TestRunner::Playwright));
    assert!(!frameworks.contains(crate::codebase::test_discovery::TestRunner::Dotnet));
}

#[test]
fn same_config_path_normalizes_relative_paths_and_preserves_optionality() {
    let root = Path::new("/repo");

    assert!(same_config_path(
        root,
        Some(Path::new("config/../no-mistakes.yml")),
        Some(Path::new("/repo/no-mistakes.yml")),
    ));
    assert!(same_config_path(root, None, None));
    assert!(!same_config_path(
        root,
        Some(Path::new("no-mistakes.yml")),
        None,
    ));
}

#[test]
fn filter_cache_keys_ignore_order_and_duplicates() {
    let left = vec![
        "src/**".to_string(),
        "tests/**".to_string(),
        "src/**".to_string(),
    ];
    let right = vec!["tests/**".to_string(), "src/**".to_string()];
    assert_eq!(
        canonical_filter_key(&left).unwrap(),
        canonical_filter_key(&right).unwrap()
    );
}

#[test]
fn report_caches_call_each_analyzer_once_per_canonical_key() {
    let key = canonical_filter_key(&[
        "src/**".to_string(),
        "tests/**".to_string(),
        "src/**".to_string(),
    ])
    .unwrap();
    let equivalent_key =
        canonical_filter_key(&["tests/**".to_string(), "src/**".to_string()]).unwrap();

    for domain in ["queue", "server"] {
        let plain_calls = Cell::new(0);
        let indexed_calls = Cell::new(0);
        let mut plain = HashMap::new();
        let mut indexed = HashMap::new();

        for traversal in [false, false, true, true] {
            let report = cached_analysis(
                &mut plain,
                &mut indexed,
                if traversal { &equivalent_key } else { &key },
                traversal,
                || {
                    plain_calls.set(plain_calls.get() + 1);
                    Ok(format!("{domain}-plain"))
                },
                || {
                    indexed_calls.set(indexed_calls.get() + 1);
                    Ok(format!("{domain}-indexed"))
                },
            )
            .unwrap();
            match (traversal, report) {
                (false, CachedAnalysis::Plain(report)) => {
                    assert_eq!(report, &format!("{domain}-plain"));
                }
                (true, CachedAnalysis::Indexed(report)) => {
                    assert_eq!(report, &format!("{domain}-indexed"));
                }
                _ => panic!("{domain} selected the wrong analyzer"),
            }
        }

        assert_eq!(plain_calls.get(), 1, "{domain} plain analyzer");
        assert_eq!(indexed_calls.get(), 1, "{domain} indexed analyzer");
    }
}

#[test]
fn omitted_and_explicit_automatic_paths_share_one_scope() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/forbidden-dependencies-passes/fixture"),
    );
    let options: super::AnalyzeProjectOptions = serde_json::from_value(serde_json::json!({
        "root": root,
        "reports": [
            { "type": "check" },
            {
                "type": "check",
                "config": ".no-mistakes.yml",
                "tsconfig": "tsconfig.json"
            }
        ]
    }))
    .unwrap();

    let context = super::AnalyzeProjectContext::prepare(&options).unwrap();
    assert_eq!(context.scopes.len(), 1);
    assert_eq!(context.scope_aliases.len(), 2);
}
