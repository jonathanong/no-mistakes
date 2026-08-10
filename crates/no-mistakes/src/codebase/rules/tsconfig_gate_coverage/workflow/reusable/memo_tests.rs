use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

fn scanned_projects(trigger_config: &str) -> BTreeSet<String> {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                &format!(
                    "on:\n{trigger_config}jobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n"
                ),
            ),
            document(
                ".github/workflows/callee.yml",
                "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project app/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);
    let inputs = ProjectSourceInputs::from([(
        "app/tsconfig.json".to_string(),
        BTreeSet::from(["app/src/index.ts".to_string()]),
    )]);

    collect_ci_projects_with_stats(&workflows, &tracked, &inputs).0
}

#[test]
fn direct_activation_memoization_is_scoped_to_each_trigger_event() {
    // The first event excludes the project while the second includes it. The
    // inverse proves coverage does not depend on which event happens to scan first.
    for triggers in [
        "  pull_request:\n    paths: ['other/**']\n  push:\n    paths: ['app/**']\n",
        "  pull_request:\n    paths: ['app/**']\n  push:\n    paths: ['other/**']\n",
    ] {
        assert_eq!(
            scanned_projects(triggers),
            BTreeSet::from(["app/tsconfig.json".to_string()])
        );
    }
}
