use super::*;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

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
    crate::test_support::without_hygiene_advisory_json(
        serde_json::from_str(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                options.to_string(),
            ))
            .unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn construction_target_audit_preserves_suppressed_channel() {
    for config in [".no-mistakes.yml", "report.yml", "advise.yml", "ignore.yml"] {
        for effort in [None, Some("low"), Some("max")] {
            let value = report(config, effort);
            assert_eq!(
                value["suppressed"].as_array().unwrap().len(),
                if config == "ignore.yml" { 0 } else { 2 },
                "{config} {effort:?}: {value}"
            );
        }
    }
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
    let mut changed_class_site = constructor.clone();
    changed_class_site.target = site.target;
    // A class proof belongs to the exact prepared occurrence, not an arbitrary
    // alternative set installed later at the same source offset.
    assert!(super::super::analysis::unresolved_constructor(
        &graph,
        &changed_class_site
    ));
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
