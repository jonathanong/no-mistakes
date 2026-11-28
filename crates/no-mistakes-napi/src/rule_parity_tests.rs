use std::path::PathBuf;

fn coverage_owner_fixture() -> PathBuf {
    no_mistakes::test_support::rule_fixture_root("coverage-producer-assertion-owner")
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn existing_graph_projection_is_unchanged_when_the_rule_is_enabled() {
    let root = coverage_owner_fixture();
    let query = |config: &str, relationships: &[&str]| {
        let input = serde_json::json!({"root":root,"config":root.join(config),"reports":[
            {"type":"dependents","files":["src/production.mts"],"relationships":relationships},
            {"type":"check"}
        ]});
        let result: serde_json::Value =
            serde_json::from_str(&crate::analyze_project_json_impl(input).unwrap()).unwrap();
        result["reports"][0]["result"].clone()
    };
    for relationships in [&["call", "import"][..], &["call"][..], &[][..]] {
        assert_eq!(
            query("disabled.yml", relationships),
            query(".no-mistakes.yml", relationships)
        );
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn generator_contracts_do_not_emit_executable_ownership_edges() {
    let root = coverage_owner_fixture();
    let input = serde_json::json!({
        "root": root, "config": root.join("generators-report.yml"),
        "reports": [{ "type": "dependencies", "includeSymbols": true,
            "files": ["src/producers.mts#generatorContract", "src/producers.mts#generatorContractOwner"],
            "relationships": ["assertion-owner"] }]
    });
    let report: serde_json::Value = no_mistakes::test_support::without_hygiene_advisory_json(
        serde_json::from_str(&crate::analyze_project_json_impl(input).unwrap()).unwrap(),
    );
    assert_eq!(
        report["reports"][0]["result"]["files"],
        serde_json::json!([]),
        "{report}"
    );
}

#[test]
fn native_json_projects_the_opt_in_assertion_owner_relationship() {
    let options = serde_json::json!({
        "root": coverage_owner_fixture(), "files": ["src/producers.mts#crossProcess"],
        "relationships": ["assertion-owner"], "includeSymbols": true, "projection": "graph"
    });
    let output: serde_json::Value =
        serde_json::from_str(&crate::dependencies_json_impl(options).unwrap()).unwrap();
    assert_eq!(
        output["roots"],
        serde_json::json!(["src/producers.mts#crossProcess"])
    );
    assert!(
        output["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["file"] == "src/assertions.mts"
                && node["symbol"] == "assertResult"
                && node["depth"] == 1
                && node["via"] == serde_json::json!(["assertion-owner"])),
        "{output}"
    );
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn builtin_ownership_does_not_change_ordinary_query_reports() {
    let saved =
        no_mistakes::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let materialized = no_mistakes::test_support::materialize_saved_fixture(&saved);
    let root = materialized.path();
    let query = |config: &str| {
        no_mistakes::ast::begin_parse_count(root);
        let input = serde_json::json!({"root":root,"config":root.join(config),"reports":[
            {"type":"dependents","files":["src/owner.mts"],"relationships":["call"]},
            {"type":"callSites","file":"src/owner.mts","exportName":"owner"},
            {"type":"check"}
        ]});
        let report: serde_json::Value =
            serde_json::from_str(&crate::analyze_project_json_impl(input).unwrap()).unwrap();
        let parses = no_mistakes::ast::finish_parse_count(root);
        assert!(!parses.is_empty(), "{report}");
        (report["reports"].as_array().unwrap()[..2].to_vec(), parses)
    };
    assert_eq!(query("disabled.yml"), query("continuations.yml"));
}

#[test]
fn native_queries_expose_builtin_callback_ownership_edges() {
    let root =
        no_mistakes::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = serde_json::json!({
        "root":root,
        "files":["src/continuations.mts#promiseImported"],
        "relationships":["assertion-owner"],"includeSymbols":true
    });
    let report: serde_json::Value =
        serde_json::from_str(&crate::dependencies_json_impl(input).unwrap()).unwrap();
    assert!(
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["file"] == "src/owner.mts"
                && node["symbol"] == "owner"
                && node["via"] == serde_json::json!(["assertion-owner"])),
        "{report}"
    );
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn builtin_ownership_projection_reuses_prepared_sources() {
    let saved =
        no_mistakes::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let materialized = no_mistakes::test_support::materialize_saved_fixture(&saved);
    let root = materialized.path();
    no_mistakes::ast::begin_parse_count(root);
    let input = serde_json::json!({"root":root,"config":root.join("continuations.yml"),"reports":[
        {"type":"dependents","files":["src/owner.mts"],"relationships":["call"]},
        {"type":"check"}
    ]});
    crate::analyze_project_json_impl(input).unwrap();
    let parses = no_mistakes::ast::finish_parse_count(root);
    assert!(!parses.is_empty());
    assert!(parses.values().all(|count| *count == 1), "{parses:?}");
}

#[test]
fn root_mock_boundaries_have_node_api_parity() {
    let fixture = no_mistakes::test_support::rule_fixture_root("forbidden-calls");
    let base = fixture.parent().unwrap();
    for (name, expected) in [
        ("full", 0),
        ("dynamic-bypass", 1),
        ("real", 1),
        ("factory", 1),
        ("root-isolation", 1),
        ("initializer", 1),
        ("initializer-dependency", 1),
        ("manual", 3),
    ] {
        let fixture = if name == "manual" {
            "mock-manual/fixture"
        } else {
            "mock-boundaries/fixture"
        };
        let input = crate::options::test_json_arg(serde_json::json!({
            "root": base.join(fixture),
            "config": base.join(format!("configs/mock-boundaries-{name}.yml")),
        }));
        let report: serde_json::Value =
            serde_json::from_str(&crate::check_json_impl(input).unwrap()).unwrap();
        #[cfg(feature = "test-instrumentation")]
        {
            let aggregate: serde_json::Value = serde_json::from_str(
                &crate::analyze_project_json_impl(crate::options::test_json_arg(
                    serde_json::json!({
                        "root": base.join(fixture),
                        "config": base.join(format!("configs/mock-boundaries-{name}.yml")),
                        "reports": [{"type": "check"}],
                    }),
                ))
                .unwrap(),
            )
            .unwrap();
            assert_eq!(aggregate["reports"][0]["result"], report, "{name}");
        }
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected,
            "{name}: {report}"
        );
    }
}
