use super::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn fixture(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for part in parts {
        path.push(part);
    }
    crate::codebase::ts_resolver::normalize_path(&path)
}

fn parse_json(value: String) -> Value {
    serde_json::from_str(&value).unwrap()
}

fn report_results(value: &Value) -> Vec<Value> {
    value["reports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|report| report["result"].clone())
        .collect()
}

#[test]
fn production_dispatch_has_no_standalone_wrappers_or_placeholder_bails() {
    let sources = [
        (
            "analyze_project.rs",
            include_str!("../../analyze_project.rs"),
        ),
        ("context.rs", include_str!("../context.rs")),
        ("dispatch.rs", include_str!("../dispatch.rs")),
    ];
    let standalone_wrappers = [
        "symbols_json_impl",
        "playwright_check_json_impl",
        "playwright_edges_json_impl",
        "playwright_related_json_impl",
        "playwright_tests_json_impl",
        "flow_json_impl",
        "effects_json_impl",
        "rsc_callers_json_impl",
        "queues_json_impl",
        "queue_edges_json_impl",
        "queue_related_json_impl",
        "queue_check_json_impl",
        "server_routes_json_impl",
        "server_route_list_json_impl",
        "server_route_edges_json_impl",
        "server_route_related_json_impl",
        "server_contracts_json_impl",
        "react_analyze_json_impl",
        "react_check_json_impl",
        "react_usages_json_impl",
        "check_json_impl",
    ];
    for (name, source) in sources {
        for wrapper in standalone_wrappers {
            assert!(
                !source.contains(wrapper),
                "{name} must dispatch through prepared facts, not `{wrapper}`"
            );
        }
        assert!(!source.contains("TODO"), "{name} contains a TODO fallback");
        assert!(
            !source.contains("does not yet have a prepared shared-context runner"),
            "{name} contains a prepared-runner fallback"
        );
    }
}

#[test]
fn playwright_prepared_views_share_one_parse_per_indexable_file() {
    let source = fixture(&["fixtures", "parser-count", "playwright"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "playwrightCheck", "id": "same-check" },
                { "type": "playwrightEdges", "id": "same-edges" },
                {
                    "type": "playwrightTests",
                    "id": "distinct-policy",
                    "files": ["app/page.tsx"],
                    "allowSkippedTests": true
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let output = parse_json(output);

    assert_eq!(output["reports"].as_array().unwrap().len(), 3);
    assert!(
        !counts.is_empty(),
        "fixture must exercise Playwright parsing"
    );
    // Distinct report policies keep separate views, but their union still derives all
    // report facts from the invocation's single parser pass.
    let expected = crate::codebase::ts_source::discover_files(&root, &[])
        .into_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .collect::<Vec<_>>();
    assert_eq!(counts.len(), expected.len(), "{counts:#?}");
    for path in expected {
        assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
    }
}

#[test]
fn explicit_config_playwright_scope_reuses_canonical_manifest_and_sources() {
    let source = fixture(&["fixtures", "parser-count", "playwright"]);
    let directory = crate::test_support::materialize_saved_fixture(&source);
    let root = directory.path().canonicalize().unwrap();
    let config = root.join(".no-mistakes.yml");
    let standalone = parse_json(
        crate::napi_api::playwright_check_json_impl(
            json!({ "root": root, "config": config }).to_string(),
        )
        .unwrap(),
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        parse_json(
            analyze_project_json_impl(
                json!({
                    "root": root,
                    "reports": [{
                        "type": "playwrightCheck",
                        "config": config
                    }]
                })
                .to_string(),
            )
            .unwrap(),
        )
    };

    assert_eq!(output["reports"][0]["result"], standalone);
    let source_reads = observer.source_read_snapshot();
    assert_eq!(source_reads[&config], 1, "{source_reads:#?}");
    assert!(
        source_reads.values().all(|reads| *reads == 1),
        "{source_reads:#?}"
    );
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 2, "{work:#?}");
    assert_eq!(work["manifest.parses"], 2, "{work:#?}");
    assert_eq!(
        work.get("manifest.cache_hits").copied().unwrap_or_default(),
        0,
        "{work:#?}"
    );
}

#[test]
fn per_report_config_scopes_match_standalone_check_results() {
    let directory = crate::test_support::materialize_gitignore_fixture("integration-aggregate");
    crate::test_support::git_init(directory.path());
    crate::test_support::git_add_all(directory.path());
    let root = directory.path();
    let explicit_config = root.join("explicit.no-mistakes.yml");
    let standalone = [
        parse_json(crate::napi_api::check_json_impl(json!({ "root": root }).to_string()).unwrap()),
        parse_json(
            crate::napi_api::check_json_impl(
                json!({ "root": root, "config": explicit_config }).to_string(),
            )
            .unwrap(),
        ),
    ];

    let output = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [
                    { "type": "check", "id": "automatic" },
                    {
                        "type": "check",
                        "id": "explicit",
                        "config": explicit_config
                    }
                ]
            })
            .to_string(),
        )
        .unwrap(),
    );

    assert_eq!(report_results(&output), standalone);
    assert_ne!(standalone[0]["integration"], standalone[1]["integration"]);
}

#[test]
fn equivalent_relative_and_absolute_roots_share_one_analysis_scope() {
    let root = fixture(&["test-cases", "codebase-analysis", "simple", "fixture"]);
    let cwd = std::env::current_dir().unwrap();
    let cwd_components = cwd.components().collect::<Vec<_>>();
    let root_components = root.components().collect::<Vec<_>>();
    let common = cwd_components
        .iter()
        .zip(&root_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative_root = PathBuf::new();
    for _ in common..cwd_components.len() {
        relative_root.push("..");
    }
    for component in &root_components[common..] {
        relative_root.push(component.as_os_str());
    }
    assert_eq!(
        crate::codebase::ts_resolver::normalize_path(&cwd.join(&relative_root)),
        root,
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [
                    {
                        "type": "dependencies",
                        "id": "relative",
                        "root": relative_root,
                        "files": ["a.mts"],
                        "relationships": ["import"]
                    },
                    {
                        "type": "dependencies",
                        "id": "absolute",
                        "root": root,
                        "files": ["a.mts"],
                        "relationships": ["import"]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap()
    };
    let output = parse_json(output);
    let results = report_results(&output);

    assert_eq!(results[0], results[1]);
    let work = observer.snapshot().work;
    assert_eq!(work["analysis.requests"], 1, "{work:#?}");
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["manifest.parses"], 2, "{work:#?}");
    assert_eq!(work.get("graph.builds").copied().unwrap_or_default(), 0);
    assert_eq!(work["traversal.computations"], 1, "{work:#?}");
    assert_eq!(work["traversal.reuses"], 1, "{work:#?}");
}

#[test]
fn inherited_and_report_relative_tsconfigs_anchor_to_their_own_roots() {
    let top_root = fixture(&["fixtures", "performance"]);
    let report_root = top_root.join("core-analysis");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(
            json!({
                "root": top_root,
                "tsconfig": "core-analysis/tsconfig.json",
                "reports": [
                    {
                        "type": "dependencies",
                        "id": "inherited",
                        "root": report_root,
                        "files": ["src/app.tsx"],
                        "relationships": ["import"]
                    },
                    {
                        "type": "dependencies",
                        "id": "report-relative",
                        "root": report_root,
                        "tsconfig": "tsconfig.json",
                        "files": ["src/app.tsx"],
                        "relationships": ["import"]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap()
    };
    let results = report_results(&parse_json(output));

    assert_eq!(results[0], results[1]);
    let work = observer.snapshot().work;
    assert_eq!(work["analysis.requests"], 1, "{work:#?}");
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
}

#[test]
fn inherited_and_report_relative_configs_anchor_to_their_own_roots() {
    let top_root = fixture(&["fixtures", "codebase", "forbidden-playwright-cached-error"]);
    let report_root = top_root.join("fixture");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(
            json!({
                "root": top_root,
                "config": "fixture/route.no-mistakes.yml",
                "reports": [
                    {
                        "type": "check",
                        "id": "inherited",
                        "root": report_root
                    },
                    {
                        "type": "check",
                        "id": "report-relative",
                        "root": report_root,
                        "config": "route.no-mistakes.yml"
                    }
                ]
            })
            .to_string(),
        )
        .unwrap()
    };
    let results = report_results(&parse_json(output));

    assert_eq!(results[0], results[1]);
    let work = observer.snapshot().work;
    assert_eq!(work["analysis.requests"], 1, "{work:#?}");
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
}

#[test]
fn nested_root_report_override_succeeds_with_the_top_level_snapshot_scope() {
    let root = fixture(&["fixtures", "parser-count", "rsc"]);
    let nested_root = root.join("app");
    let output = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [{
                    "type": "dependencies",
                    "root": nested_root,
                    "files": ["page.tsx"],
                    "relationships": ["import"]
                }]
            })
            .to_string(),
        )
        .unwrap(),
    );
    let files = output["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files.iter().any(|file| file["path"] == "ServerWidget.tsx"));
    assert!(Path::new(&nested_root).starts_with(&root));
}

#[test]
fn playwright_preparation_surfaces_settings_and_fact_plan_errors() {
    let root = fixture(&["fixtures", "parser-count", "playwright"]);

    let settings_error = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{
                "type": "playwrightCheck",
                "config": "missing.no-mistakes.yml"
            }]
        })
        .to_string(),
    )
    .expect_err("an explicit missing no-mistakes config must fail preparation");
    assert!(settings_error
        .to_string()
        .contains("config file does not exist"));

    let fact_plan_error = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{
                "type": "playwrightCheck",
                "playwrightConfig": ["missing.playwright.config.ts"]
            }]
        })
        .to_string(),
    )
    .expect_err("an explicit missing Playwright config must fail fact planning");
    assert!(fact_plan_error
        .to_string()
        .contains("Playwright config does not exist"));
}
