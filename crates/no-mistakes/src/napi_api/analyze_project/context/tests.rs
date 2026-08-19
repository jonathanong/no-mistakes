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
        let plain = ReportCache::new(HashMap::new());
        let indexed = ReportCache::new(HashMap::new());

        for traversal in [false, false, true, true] {
            let report = cached_analysis(
                &plain,
                &indexed,
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
                    assert_eq!(report, format!("{domain}-plain"));
                }
                (true, CachedAnalysis::Indexed(report)) => {
                    assert_eq!(report, format!("{domain}-indexed"));
                }
                _ => panic!("{domain} selected the wrong analyzer"),
            }
        }

        assert_eq!(plain_calls.get(), 1, "{domain} plain analyzer");
        assert_eq!(indexed_calls.get(), 1, "{domain} indexed analyzer");
    }
}

#[test]
fn omitted_automatic_and_explicit_tsconfig_use_separate_scopes() {
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
    assert_eq!(context.scopes.len(), 2);
    assert_eq!(context.scope_aliases.len(), 2);
    let mut automatic_modes = context
        .scopes
        .keys()
        .map(|scope| scope.automatic_tsconfig)
        .collect::<Vec<_>>();
    automatic_modes.sort();
    assert_eq!(automatic_modes, vec![false, true]);
}

#[test]
fn command_report_rejects_unknown_type() {
    let request = super::super::types::AnalyzeReportRequest {
        id: None,
        report_type: "missing".to_string(),
        options: serde_json::Map::new(),
    };
    let options = super::super::types::AnalyzeProjectOptions {
        root: None,
        tsconfig: None,
        config: None,
        filters: Vec::new(),
        reports: Vec::new(),
    };
    let error = super::run_command_report(&request, &options).unwrap_err();
    assert!(error
        .to_string()
        .contains("unknown analyzeProject report type: missing"));
}
