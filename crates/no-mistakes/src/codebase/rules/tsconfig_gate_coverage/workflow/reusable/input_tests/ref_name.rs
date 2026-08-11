use super::*;

#[test]
fn exact_ref_name_is_forwarded_to_reusable_workflow_conditions() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on:\n  push:\n    branches: [main]\njobs:\n  direct-matching:\n    if: github.ref_name == 'main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-matching/tsconfig.json\n  direct-mismatching:\n    if: github.ref_name != 'main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-mismatching/tsconfig.json\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      branch: '${{ github.ref_name }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      branch: {type: string, required: true}\njobs:\n  matching:\n    if: inputs.branch == 'main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project matching/tsconfig.json\n  mismatching:\n    if: inputs.branch != 'main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project mismatching/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "direct-matching/tsconfig.json".to_string(),
        "direct-mismatching/tsconfig.json".to_string(),
        "matching/tsconfig.json".to_string(),
        "mismatching/tsconfig.json".to_string(),
    ]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "direct-matching/tsconfig.json".to_string(),
            "matching/tsconfig.json".to_string(),
        ])
    );
}
