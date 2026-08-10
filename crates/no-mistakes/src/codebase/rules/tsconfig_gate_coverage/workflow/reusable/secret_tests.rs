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
fn reusable_secret_availability_survives_only_explicit_or_known_inheritance() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/absent-root.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/absent-intermediate.yml\n    secrets:\n      other: '${{ secrets.OTHER }}'\n",
            ),
            document(
                ".github/workflows/absent-intermediate.yml",
                "on:\n  workflow_call:\n    secrets:\n      other: {required: true}\njobs:\n  call:\n    uses: ./.github/workflows/absent-leaf.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/absent-leaf.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p absent/tsconfig.json\n",
            ),
            document(
                ".github/workflows/explicit-root.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/explicit-intermediate.yml\n    secrets:\n      token: '${{ secrets.TOKEN }}'\n",
            ),
            document(
                ".github/workflows/explicit-intermediate.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  call:\n    uses: ./.github/workflows/explicit-leaf.yml\n    secrets:\n      token: '${{ secrets.token }}'\n",
            ),
            document(
                ".github/workflows/explicit-leaf.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p explicit/tsconfig.json\n",
            ),
            document(
                ".github/workflows/inherited-root.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/inherited-intermediate.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/inherited-intermediate.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  call:\n    uses: ./.github/workflows/inherited-leaf.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/inherited-leaf.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p inherited/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "absent/tsconfig.json".to_string(),
        "explicit/tsconfig.json".to_string(),
        "inherited/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "explicit/tsconfig.json".to_string(),
            "inherited/tsconfig.json".to_string(),
        ])
    );
}
