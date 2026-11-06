use super::*;

#[test]
fn dynamic_import_reachability_cuts_mocked_intermediaries_per_test() {
    let root = fixture(
        "codebase-analysis",
        "test-no-unmocked-dynamic-imports-mock-cut",
    );
    for (config, expected) in [
        (".no-mistakes-mocked.yml", None),
        (".no-mistakes-unmocked.yml", Some("src/leaf.mts")),
        (".no-mistakes-alternate.yml", Some("src/leaf.mts")),
        (".no-mistakes-direct.yml", Some("src/direct-entry.mts")),
        (".no-mistakes-mixed.yml", Some("src/leaf.mts")),
    ] {
        let output = run(&[
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            config,
            "--json",
        ]);
        let body = stdout(&output);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        let findings = value["rules"].as_array().unwrap();
        assert_eq!(
            findings.len(),
            usize::from(expected.is_some()),
            "{config}: {body}"
        );
        if let Some(target) = expected {
            assert_eq!(findings[0]["target"], target, "{config}: {body}");
        }
    }
}
