use crate::codebase::rules::{run_check, RuleFinding};
use std::path::PathBuf;

mod gap;
mod selector_shape;

pub(super) fn fixture(case: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-exports-only-used-by-tests/fixture")
            .join(case),
    )
}

fn findings(case: &str) -> Vec<RuleFinding> {
    run_check(&fixture(case), None, None).unwrap()
}

#[test]
fn reports_origin_exports_through_aliases_stars_helpers_and_types() {
    let found = findings("fail");
    let actual: Vec<_> = found
        .iter()
        .map(|row| {
            format!(
                "{}:{} {}",
                row.file,
                row.line,
                row.target.as_deref().unwrap()
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            "src/flow.mts:1 createFlowProducer",
            "src/link.mts:1 createLinkPost",
            "src/request.mts:1 getRequestClientInfo",
            "src/same-file.mts:1 onlyTest",
            "src/types.mts:1 TestContract",
            "src/types.mts:2 testDefault",
        ]
    );
}

#[test]
fn production_syntactic_imports_count_and_unused_exports_are_ignored() {
    // An unused named production import must keep a mixed test consumer quiet.
    assert!(findings("pass").is_empty());
}

#[test]
fn supports_file_line_and_next_line_suppression() {
    assert!(findings("suppressed").is_empty());
}

#[test]
fn computed_and_unresolved_imports_do_not_silently_pass() {
    let found = findings("uncertain");
    assert!(found.iter().any(|row| row.file == "src/dynamic.mts"));
    assert!(found.iter().any(|row| row.file == "src/unresolved.mts"));
    assert!(found
        .iter()
        .any(|row| row.target.as_deref() == Some("uncertain")));
}

#[test]
fn opaque_literal_selectors_only_make_their_target_modules_uncertain() {
    let found = findings("bounded");
    assert_eq!(found.len(), 3);
    assert!(found
        .iter()
        .any(|row| row.file == "src/definite.mts" && row.target.as_deref() == Some("onlyTests")));
    for file in ["src/namespace.mts", "src/dynamic.mts"] {
        assert!(found
            .iter()
            .any(|row| row.file == file && row.target.as_deref() == Some("unanalyzable")));
    }
}

#[test]
fn malformed_importer_diagnostics_preserve_known_test_only_exports() {
    let found = findings("malformed");
    assert!(found.iter().any(|row| row.file == "tests/broken.mts"));
    assert!(found
        .iter()
        .any(|row| row.target.as_deref() == Some("value")));
}

#[test]
fn unsupported_commonjs_and_import_equals_are_explicit_at_their_own_lines() {
    let found = findings("unsupported-modules");
    for file in ["src/common.cjs", "src/consumer.cts", "src/assignment.cts"] {
        assert!(
            found.iter().any(|row| row.file == file && row.line == 2),
            "{found:?}"
        );
    }
    assert!(found
        .iter()
        .any(|row| row.target.as_deref() == Some("value")));
    let root = fixture("unsupported-modules");
    for (policy, advisories) in [("advise", true), ("ignore", false)] {
        let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(format!("{policy}.yml")),
            })
            .to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), 1);
        assert_eq!(report["rules"][0]["target"], "value");
        assert_eq!(
            !report["advisories"].as_array().unwrap().is_empty(),
            advisories
        );
    }
}

#[test]
fn reexport_cycles_without_terminal_importers_are_ignored() {
    assert!(findings("cycle").is_empty());
}

#[test]
fn rejects_missing_explicit_scopes() {
    for (case, expected) in [
        ("missing-production", "productionRoots"),
        ("missing-tests", "testGlobs"),
        ("blank-root", "empty strings"),
        ("blank-tests", "testGlobs"),
        ("invalid-glob", "invalid glob"),
    ] {
        let error = run_check(&fixture(case), None, None).unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}

#[test]
fn uncertainty_respects_shared_analysis_policy() {
    let root = fixture("uncertain");
    for (config, channel) in [("advise.yml", "advisories"), ("ignore.yml", "rules")] {
        let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(config)
            })
            .to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), 1);
        assert_eq!(report["rules"][0]["target"], "uncertain");
        if config == "advise.yml" {
            assert!(!report[channel].as_array().unwrap().is_empty());
        } else {
            assert!(report["advisories"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
fn unrelated_uncertainty_never_hides_a_known_violation_under_any_policy() {
    let root = fixture("policy-isolation");
    for policy in ["report", "advise", "ignore"] {
        let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(format!("{policy}.yml")),
            })
            .to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&report).unwrap();
        let rules = report["rules"].as_array().unwrap();
        let definite: Vec<_> = rules
            .iter()
            .filter(|row| row["target"] != "unanalyzable")
            .collect();
        assert_eq!(definite.len(), 1, "{policy}: {report}");
        assert_eq!(definite[0]["file"], "src/value.mts");
        assert_eq!(definite[0]["target"], "testOnly");
        let uncertainties: Vec<_> = rules
            .iter()
            .filter(|row| row["target"] == "unanalyzable")
            .collect();
        let advisories = report["advisories"].as_array().unwrap();
        assert_eq!(
            !uncertainties.is_empty(),
            policy == "report",
            "{policy}: {report}"
        );
        assert_eq!(
            !advisories.is_empty(),
            policy == "advise",
            "{policy}: {report}"
        );
        for row in uncertainties.into_iter().chain(advisories) {
            assert_eq!(row["file"], "other/dynamic.mts");
            assert_eq!(row["target"], "unanalyzable");
        }
    }
}

#[test]
fn missing_prepared_importer_facts_are_not_an_empty_importer_set() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::test_support::from_raw_maps;
    let root = fixture("fail");
    let file = root.join("tests/link.test.mts");
    // A declared inventory entry without its facts models an owner preparation
    // failure; it must remain visible to the rule as uncertainty.
    let facts = CheckFactMap {
        files: vec![file.clone()],
        ..Default::default()
    };
    let graph = from_raw_maps(root, Default::default(), Default::default());
    let found = super::traversal::incomplete_sources(&graph, &facts);
    assert_eq!(
        found,
        vec![(file, 1, "missing prepared importer facts".to_owned())]
    );
}

#[test]
fn shared_graph_file_edges_do_not_widen_symbol_importers() {
    use crate::codebase::dependencies::graph::{test_support::from_typed_maps, EdgeKind, NodeId};
    let root = fixture("fail");
    let file = root.join("src/link.mts");
    let test = root.join("tests/link.test.mts");
    let unrelated = root.join("src/production-consumer.mts");
    let symbol = NodeId::symbol(&file, "createLinkPost");
    let reverse = crate::fx::FxHashMap::from_iter([
        (
            symbol.clone(),
            vec![
                (NodeId::file(&test), EdgeKind::SymbolImport),
                (NodeId::file(&file), EdgeKind::Import),
            ],
        ),
        (
            NodeId::file(&file),
            vec![(NodeId::file(&unrelated), EdgeKind::Import)],
        ),
    ]);
    let graph = from_typed_maps(root, Default::default(), reverse);
    let consumers = super::traversal::consumers(&graph, &file, "createLinkPost");
    assert_eq!(consumers.files.into_iter().collect::<Vec<_>>(), vec![test]);
    assert!(consumers.unknown.is_empty());
}

#[test]
fn custom_message_prefixes_definite_and_uncertain_diagnostics() {
    let root = fixture("bounded");
    let found = run_check(&root, Some(&root.join("custom.yml")), None).unwrap();
    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|finding| finding
        .message
        .starts_with("Keep production APIs focused: ")));
    assert!(found
        .iter()
        .any(|finding| finding.target.as_deref() == Some("onlyTests")));
    assert!(found
        .iter()
        .any(|finding| finding.target.as_deref() == Some("unanalyzable")));
}
