use serde_yaml::Value;

#[test]
fn ci_checks_workspace_complexity_without_requiring_coverage() {
    let workflow: Value =
        serde_yaml::from_str(include_str!("../../../.github/workflows/ci.yml")).unwrap();
    let steps = workflow["jobs"]["tests"]["steps"].as_sequence().unwrap();
    let command = steps
        .iter()
        .find(|step| step["name"].as_str() == Some("Enforce cyclomatic complexity threshold"))
        .unwrap()["run"]
        .as_str()
        .unwrap();
    // src/tests contains production planners that the LCOV defaults exclude.
    // Optimistic missing coverage must therefore enforce CC independently.
    assert_eq!(
        command.split_whitespace().collect::<Vec<_>>(),
        [
            "cargo",
            "crap",
            "--workspace",
            "--missing",
            "optimistic",
            "--fail-above",
            "--threshold",
            "30",
            "--summary",
        ]
    );
}
