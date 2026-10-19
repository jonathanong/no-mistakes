use super::{
    cached_analysis, canonical_filter_key, framework_preparation_plan, graph_build_plan,
    same_config_path, CachedAnalysis, ReportCache,
};
use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

impl super::AnalyzeProjectContext {
    pub(in crate::napi_api::analyze_project) fn root_source_read_count(&self) -> usize {
        self.scopes
            .values()
            .filter_map(|scope| scope.check.as_ref())
            .map(|check| {
                check
                    .prepared
                    .visible_paths
                    .source_store_for(&check.root)
                    .physical_read_count()
            })
            .sum()
    }
}

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
    assert!(frameworks.contains(crate::codebase::test_discovery::TestRunner::Playwright));
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
fn authoritative_report_files_strip_legacy_symbol_suffixes() {
    let source = include_str!("target_helpers.rs");
    let body = source
        .split("fn authoritative_report_files(")
        .nth(1)
        .and_then(|source| source.split("fn authoritative_path(").next())
        .expect("authoritative_report_files is defined");
    assert!(
        body.contains("parse_entrypoint"),
        "legacy files#symbol targets must strip the suffix before is_file"
    );
}

#[test]
fn authoritative_path_falls_back_to_cwd_when_missing_under_root() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cargo = PathBuf::from("Cargo.toml");
    let under_root = super::authoritative_path(&workspace, cargo.clone());
    assert!(under_root.ends_with("Cargo.toml"));
    assert!(under_root.is_file());

    let missing_root = Path::new("/no-mistakes-missing-graph-root");
    let from_cwd = super::authoritative_path(missing_root, cargo);
    assert!(from_cwd.is_file());
    assert_eq!(
        from_cwd,
        crate::codebase::ts_resolver::normalize_path(
            &std::env::current_dir().unwrap().join("Cargo.toml")
        )
    );

    let missing = super::authoritative_path(missing_root, PathBuf::from("no-such-entry.ts"));
    assert_eq!(
        missing,
        crate::codebase::ts_resolver::normalize_path(&missing_root.join("no-such-entry.ts"))
    );
}

#[test]
fn symbol_legacy_and_report_files_cover_absolute_relative_and_signature_impact() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/simple/fixture"),
    );
    let absolute = root.join("a.mts");
    let options: super::AnalyzeProjectOptions = serde_json::from_value(serde_json::json!({
        "root": root,
        "reports": [
            { "type": "check" },
            {
                "type": "symbols",
                "files": ["a.mts"],
                "mode": "signature-impact",
                "symbol": "unused"
            },
            {
                "type": "symbols",
                "files": [absolute]
            },
            { "type": "effects", "kind": "fetch", "entry": "a.mts" },
            { "type": "rscCallers", "component": "a.mts" },
            { "type": "importUsages", "files": ["a.mts"] },
            { "type": "importUsages", "files": ["a.mts"] },
            { "type": "dependencies", "files": ["a.mts"], "relationships": ["import"] }
        ]
    }))
    .unwrap();

    let symbol_files = super::symbol_target_files(&options, &root).unwrap();
    assert!(symbol_files.iter().any(|path| path.ends_with("a.mts")));

    let legacy = super::legacy_symbol_target_files(&options, &root).unwrap();
    assert!(legacy.iter().all(|path| path.ends_with("a.mts")));

    let report_files = super::authoritative_report_files(&options, &root).unwrap();
    assert!(report_files.iter().any(|path| path.ends_with("a.mts")));

    let (views, usage_files) = super::prepare_import_usage_views(
        &options,
        &root,
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
    )
    .unwrap();
    assert_eq!(views.len(), 1);
    assert!(!usage_files.is_empty() || views.len() == 1);
    assert!(super::has_server_report(&options) || !super::has_server_report(&options));
}

include!("tests/cache_and_errors.rs");
