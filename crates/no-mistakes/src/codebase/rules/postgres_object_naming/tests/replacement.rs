use super::support::messages;

#[test]
fn denied_tokens_are_accepted_only_inside_their_own_boundary_aligned_replacement() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-object-naming/fixture");
    for (scenario, expected) in [
        ("replacement-pass", 0),
        ("replacement-fail", 7),
        ("replacement-position-pass", 0),
        ("replacement-position-fail", 3),
    ] {
        let path = root.join(scenario);
        let config: serde_yaml::Value =
            serde_yaml::from_str(&std::fs::read_to_string(path.join(".no-mistakes.yml")).unwrap())
                .unwrap();
        let yaml = serde_yaml::to_string(&config["rules"][0]["options"]).unwrap();
        let catalog =
            serde_json::from_str(&std::fs::read_to_string(path.join("schema.json")).unwrap())
                .unwrap();
        let findings = messages(&yaml, catalog);
        assert_eq!(findings.len(), expected, "{scenario}: {findings:?}");
    }
}
