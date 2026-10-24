use super::*;

#[test]
fn generator_creation_and_unsupported_consumption_abstain_under_every_policy() {
    let root = fixture();
    for (policy, expected_findings, expected_advisories) in
        [("report", 7, 0), ("advise", 0, 7), ("ignore", 0, 0)]
    {
        let input = crate::napi_api::options::test_json_arg(serde_json::json!({
            "root": root, "config": root.join(format!("generators-{policy}.yml"))
        }));
        let report: serde_json::Value =
            serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected_findings,
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            expected_advisories,
            "{report}"
        );
        for finding in report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .chain(report["advisories"].as_array().unwrap())
        {
            assert!(
                finding["message"]
                    .as_str()
                    .unwrap()
                    .contains("unanalyzable"),
                "{report}"
            );
        }
    }
}

#[test]
fn generator_contracts_do_not_emit_executable_ownership_edges() {
    let root = fixture();
    let input = serde_json::json!({
        "root": root, "config": root.join("generators-report.yml"),
        "reports": [{ "type": "dependencies", "includeSymbols": true,
            "files": ["src/producers.mts#generatorContract", "src/producers.mts#generatorContractOwner"],
            "relationships": ["assertion-owner"] }]
    });
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::analyze_project_json_impl(input).unwrap()).unwrap();
    assert_eq!(
        report["reports"][0]["result"]["files"],
        serde_json::json!([]),
        "{report}"
    );
}
