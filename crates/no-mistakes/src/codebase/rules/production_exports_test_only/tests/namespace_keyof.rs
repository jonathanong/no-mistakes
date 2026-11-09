#[test]
fn namespace_keyof_helpers_keep_uncertainty_at_each_import_site() {
    let root = super::fixture("namespace-keyof-helper");
    for (config, channel, count) in [
        (".no-mistakes.yml", "advisories", 2),
        ("report.yml", "rules", 2),
        ("ignore.yml", "advisories", 0),
    ] {
        let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root": root, "config": root.join(config)}).to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&report).unwrap();
        let definite: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] != "unanalyzable")
            .collect();
        assert_eq!(definite.len(), 1, "{config}: {report}");
        assert_eq!(definite[0]["target"], "onlyTests");
        let uncertain: Vec<_> = report[channel]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] == "unanalyzable")
            .collect();
        assert_eq!(uncertain.len(), count, "{config}: {report}");
        if count > 0 {
            for file in ["src/consumer.mts", "src/other.mts"] {
                let row = uncertain.iter().find(|row| row["file"] == file).unwrap();
                assert_eq!(row["line"], 1);
                assert!(row["message"].as_str().unwrap().contains("./templates.mjs"));
                assert!(row["message"]
                    .as_str()
                    .unwrap()
                    .contains("namespace selector"));
                for name in ["processX", "processY", "processZ"] {
                    assert!(row["message"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("`{name}`")));
                }
            }
        }
    }
}
