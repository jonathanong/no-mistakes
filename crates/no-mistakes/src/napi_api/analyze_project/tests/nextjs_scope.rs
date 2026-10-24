use super::*;

#[test]
fn analyze_project_nextjs_scope_preserves_check_and_graph_batch_parity() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-facts/family-scope-nextjs"),
    );
    let analyze = |config: &str, reports: Value| {
        let output = analyze_project_value_impl(json!({
            "root": root, "config": config, "reports": reports
        }))
        .unwrap();
        serde_json::from_str::<Value>(&output).unwrap()
    };
    for (config, expected) in [
        ("graph.yml", vec!["web/src/selected.ts"]),
        ("low.yml", vec!["web/src/selected.ts"]),
        (
            "mixed.yml",
            vec!["backend/excluded.ts", "web/src/selected.ts"],
        ),
        ("low-empty.yml", vec![]),
    ] {
        let single = analyze(config, json!([{ "type": "check" }]));
        let batch = analyze(
            config,
            json!([
                { "type": "check" },
                { "type": "dependents", "files": ["web/src/selected.ts"], "relationships": ["import"] }
            ]),
        );
        let report = &single["reports"][0]["result"];
        assert_eq!(report, &batch["reports"][0]["result"]);
        let standalone: Value = serde_json::from_str(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "config": config }).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(report, &standalone);
        let cache: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["rule"] == "nextjs-no-caching")
            .collect();
        let mut actual: Vec<_> = cache
            .iter()
            .map(|finding| finding["file"].as_str().unwrap())
            .collect();
        actual.sort();
        assert_eq!(actual, expected, "{config}: {report}");
        let advisories: Vec<_> = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["rule"] == "nextjs-no-caching")
            .map(|finding| finding["file"].as_str().unwrap())
            .collect();
        let expected_advisories = if config == "low.yml" || config == "mixed.yml" {
            vec!["web/src/advisory.ts"]
        } else {
            vec![]
        };
        assert_eq!(advisories, expected_advisories, "{config}: {report}");
        assert!(!cache
            .iter()
            .any(|finding| finding["file"] == "web/src/skipped/selected.ts"));
    }
}
