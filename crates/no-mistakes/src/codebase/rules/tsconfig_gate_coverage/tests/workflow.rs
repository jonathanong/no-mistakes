use super::*;

#[test]
fn ci_scanner_credits_only_workflows_with_file_triggers() {
    let workflow = |path: &str, yaml: &str| ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    };
    let job = |project: &str| {
        format!(
            "jobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project {project}/tsconfig.json\n"
        )
    };
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow("missing.yml", &job("missing")),
            workflow("empty.yml", "on: push"),
            workflow(
                "manual.yml",
                &format!("on: workflow_dispatch\n{}", job("manual")),
            ),
            workflow(
                "scheduled.yml",
                &format!("on: schedule\n{}", job("scheduled")),
            ),
            workflow(
                "pull-request.yml",
                &format!("on: pull_request\n{}", job("pull-request")),
            ),
        ],
    };

    assert_eq!(
        ci_typechecked_projects(&workflows),
        BTreeSet::from(["pull-request/tsconfig.json".to_string()])
    );
}
