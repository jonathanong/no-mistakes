use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

#[test]
fn reusable_calls_keep_pull_request_actions_correlated_with_declared_types() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on:\n  pull_request:\n    types: [synchronize, opened]\n    paths: ['**']\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on: workflow_call\njobs:\n  opened:\n    if: github.event.action == 'opened'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p opened/tsconfig.json\n  synchronized:\n    if: github.event.action == 'synchronize'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p synchronized/tsconfig.json\n  closed:\n    if: github.event.action == 'closed'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p closed/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "opened/tsconfig.json".to_string(),
        "synchronized/tsconfig.json".to_string(),
        "closed/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "opened/tsconfig.json".to_string(),
            "synchronized/tsconfig.json".to_string(),
        ])
    );
}
