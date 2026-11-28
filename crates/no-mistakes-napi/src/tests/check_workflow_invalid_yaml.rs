use super::*;

#[test]
fn check_preserves_workflow_yaml_advisories_from_all_six_rules() {
    let root = no_mistakes::test_support::rule_fixture_root("github-actions-invalid-yaml");
    let config = root.parent().unwrap().join("configs/advise.yml");
    let options =
        crate::options::test_json_arg(json!({ "root": root, "config": config }).to_string());
    let output: serde_json::Value =
        serde_json::from_str(&check_json_impl(options).unwrap()).unwrap();
    assert_eq!(output["rules"].as_array().unwrap().len(), 6);
    let mut broken: Vec<_> = output["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["file"] == ".github/workflows/broken.yml")
        .map(|row| {
            assert_eq!(row["line"], 3);
            assert_eq!(row["target"], "unanalyzable");
            row["rule"].as_str().unwrap()
        })
        .collect();
    broken.sort_unstable();
    assert_eq!(
        broken,
        [
            "github-actions-action-timeout-pair",
            "github-actions-docker-host-ports",
            "github-actions-download-integrity",
            "github-actions-no-poll-wait-job",
            "github-actions-pr-diff-range",
            "github-actions-vitest-project-selection",
        ]
    );
}
