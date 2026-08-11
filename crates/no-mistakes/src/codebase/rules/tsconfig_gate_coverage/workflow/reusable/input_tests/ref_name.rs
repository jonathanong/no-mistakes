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

#[test]
fn exact_pull_request_base_ref_reaches_direct_and_reusable_conditions() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on:\n  pull_request:\n    types: [synchronize]\n    branches: [main]\njobs:\n  direct-matching:\n    if: github.base_ref == 'main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-matching/tsconfig.json\n  direct-mismatching:\n    if: github.base_ref != 'main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-mismatching/tsconfig.json\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      branch: '${{ github.base_ref }}'\n",
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

#[test]
fn non_pull_request_base_ref_is_the_empty_string() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  direct-empty:\n    if: github.base_ref == ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-empty/tsconfig.json\n  direct-nonempty:\n    if: github.base_ref != ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-nonempty/tsconfig.json\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      branch: '${{ github.base_ref }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      branch: {type: string, required: true}\njobs:\n  empty:\n    if: inputs.branch == ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty/tsconfig.json\n  nonempty:\n    if: inputs.branch != ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project nonempty/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "direct-empty/tsconfig.json".to_string(),
        "direct-nonempty/tsconfig.json".to_string(),
        "empty/tsconfig.json".to_string(),
        "nonempty/tsconfig.json".to_string(),
    ]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "direct-empty/tsconfig.json".to_string(),
            "empty/tsconfig.json".to_string(),
        ])
    );
}
