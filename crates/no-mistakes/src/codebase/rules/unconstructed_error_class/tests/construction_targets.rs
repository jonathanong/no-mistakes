use super::*;

fn root() -> PathBuf {
    fixture()
        .parent()
        .unwrap()
        .join("construction-targets/fixture")
}

fn report(config: &str, effort: Option<&str>) -> serde_json::Value {
    let root = root();
    let mut options = serde_json::json!({
        "root": root, "config": root.join(config), "includeSuppressed": true,
    });
    if let Some(effort) = effort {
        options["effort"] = effort.into();
    }
    serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            options.to_string(),
        ))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn resolved_constructors_and_intrinsics_preserve_definite_error_findings() {
    for config in [".no-mistakes.yml", "report.yml", "advise.yml", "ignore.yml"] {
        for effort in [None, Some("low"), Some("max")] {
            let value = report(config, effort);
            let names: Vec<_> = value["rules"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["target"].as_str().unwrap())
                .collect();
            assert_eq!(
                names,
                ["DeadError", "IntlUnusedError", "KnownCtorUnusedError"],
                "{value}"
            );
            let advisories = value["advisories"].as_array().unwrap();
            assert_eq!(
                advisories.len(),
                if config == "ignore.yml" { 0 } else { 10 },
                "{value}"
            );
            assert_eq!(
                value["suppressed"].as_array().unwrap().len(),
                if config == "ignore.yml" { 0 } else { 2 },
                "{value}"
            );
            assert!(advisories.iter().all(|row| row["target"] == "unanalyzable"));
            assert!(advisories.iter().all(|row| row["file"] != "src/main.ts"
                && row["file"] != "src/globals.ts"
                && row["file"] != "src/errors.ts"));
            if config == "report.yml" {
                assert!(value["rules"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .chain(advisories)
                    .all(|row| row["message"]
                        .as_str()
                        .unwrap()
                        .starts_with("constructor policy:")));
            }
        }
    }
}

#[test]
fn constructor_policy_matches_aggregate_api_and_preserves_ordinary_queries() {
    let root = root();
    for config in [".no-mistakes.yml", "report.yml", "advise.yml", "ignore.yml"] {
        let output =
            crate::napi_api::analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({
                    "root": root, "config": root.join(config),
                    "reports": [{"type":"check", "includeSuppressed":true}],
                })
                .to_string(),
            ))
            .unwrap();
        let project: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(project["reports"][0]["result"], report(config, None));
    }
    let analyze = |check| {
        let mut reports = vec![
            serde_json::json!({"type":"importers", "file":"src/plain.ts"}),
            serde_json::json!({"type":"callSites", "file":"src/errors.ts", "exportName":"UsedError"}),
        ];
        if check {
            reports.push(serde_json::json!({"type":"check"}));
        }
        let output =
            crate::napi_api::analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({"root":root,"reports":reports}).to_string(),
            ))
            .unwrap();
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    };
    let ordinary = analyze(false);
    let combined = analyze(true);
    assert!(ordinary.to_string().contains("main.ts"));
    for index in 0..2 {
        assert_eq!(ordinary["reports"][index], combined["reports"][index]);
    }
}

#[test]
fn consumer_pg_object_facade_and_nullish_ajv_remain_nonfailing_uncertainty() {
    let root = fixture().parent().unwrap().join("real-imports/fixture");
    let report = diagnostic_report(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        ["src/errors.ts:1 UnusedError"]
    );
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "src/data-stores/psql/runtime.mts:4 unanalyzable",
            "src/main.mts:5 unanalyzable",
            "src/main.mts:7 unanalyzable",
        ]
    );
    assert!(report["advisories"].as_array().unwrap().iter().all(|row| {
        let message = row["message"].as_str().unwrap();
        message.contains("`Pool`") || message.contains("`AjvCtor`")
    }));
}

#[test]
fn prepared_unknown_alternatives_are_not_erased_by_constructor_metadata() {
    use crate::codebase::dependencies::graph::{
        DepGraph, GraphBuildPlan, KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate,
        ResolvedCallTarget,
    };
    use crate::codebase::ts_resolver::TsConfig;
    let root = root();
    let graph = DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            paths: vec![("@app/*".into(), vec!["src/*".into()])],
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            extends: true,
            ..Default::default()
        },
    )
    .unwrap();
    let class = |namespace| {
        graph
            .class_declarations()
            .iter()
            .find(|class| {
                class.file.ends_with("src/identity.ts")
                    && class.namespace.as_deref() == Some(namespace)
            })
            .unwrap()
    };
    let a = class("A");
    let b = class("B");
    let constructor = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "A.Twin")
        .unwrap();
    assert_eq!(
        graph.constructor_class(&constructor.file, constructor.offset),
        Some(&a.node())
    );
    assert_ne!(a.node(), b.node());
    let mut site = graph
        .resolved_call_sites()
        .iter()
        .find(|site| {
            site.file.ends_with("src/main.ts") && site.source_callee == "Intl.NumberFormat"
        })
        .unwrap()
        .clone();
    assert!(!super::super::analysis::unresolved_constructor(
        &graph, &site
    ));
    // This is a prepared graph compatibility input, not a claimed parser shape.
    site.target = ResolvedCallTarget::Alternatives(
        ResolvedCallAlternatives::new(
            vec![ResolvedCallCandidate {
                target: KnownCallTarget::Global { name: "Map".into() },
                node: None,
            }],
            true,
        )
        .into(),
    );
    assert!(super::super::analysis::unresolved_constructor(
        &graph, &site
    ));
}

#[test]
fn cjs_namespace_alias_constructor_is_known_even_without_public_class_flags() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("constructor-credit/fixture");
    let report = diagnostic_report(&root);
    assert_eq!(report["rules"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
}

#[test]
fn prepared_checker_rejects_invalid_config_instead_of_reclassifying_it_as_advisory() {
    let root = root();
    let graph = crate::codebase::dependencies::graph::test_support::from_typed_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    for (name, detail) in [
        ("invalid-policy.yml", "unanalyzable"),
        ("invalid-tests.yml", "invalid type"),
        ("invalid-test-glob.yml", "glob"),
        ("invalid-filter.yml", "glob"),
    ] {
        // Prepared Rust inputs can bypass the public loader's validation.
        // These are saved malformed configurations, not fabricated TS facts.
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(&source).unwrap();
        let error = super::super::check::check_with_graph(&root, &config, &graph)
            .err()
            .expect("invalid prepared configuration must fail")
            .to_string();
        assert!(error.contains(detail), "{name}: {error}");
    }
}

#[test]
fn multiple_unseen_sources_keep_failing_with_precise_additional_file_counts() {
    let root = fixture().parent().unwrap().join("parse-counts/fixture");
    for (config, detail) in [
        ("two.yml", "and 1 other file"),
        (".no-mistakes.yml", "and 2 other files"),
    ] {
        let error = run_check(&root, Some(&root.join(config)), None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("`src/a.ts`"), "{error}");
        assert!(error.contains(detail), "{error}");
        assert!(error.contains("failed to parse"), "{error}");
    }
}
