use super::*;

#[test]
fn reusable_conditions_resolve_literal_comparisons_and_negated_parenthesized_inputs() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  enabled-input:\n    uses: ./.github/workflows/negated-disabled.yml\n    with: {enabled: true}\n  disabled-input:\n    uses: ./.github/workflows/negated-enabled.yml\n    with: {enabled: false}\n  literals:\n    uses: ./.github/workflows/literals.yml\n",
            ),
            document(
                ".github/workflows/negated-disabled.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    if: '!((inputs.enabled))'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project negated-disabled/tsconfig.json\n",
            ),
            document(
                ".github/workflows/negated-enabled.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    if: '!((inputs.enabled))'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project negated-enabled/tsconfig.json\n",
            ),
            document(
                ".github/workflows/literals.yml",
                "on: workflow_call\njobs:\n  number-disabled:\n    if: 1 == 2\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project number-disabled/tsconfig.json\n  relational-disabled:\n    if: 1 < 0\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project relational-disabled/tsconfig.json\n  string-disabled:\n    if: \"'production' == 'staging'\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project string-disabled/tsconfig.json\n  number-enabled:\n    if: 1 == 1\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project number-enabled/tsconfig.json\n  relational-enabled:\n    if: 1 > 0\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project relational-enabled/tsconfig.json\n  string-enabled:\n    if: \"'production' == 'production'\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project string-enabled/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "negated-disabled/tsconfig.json",
        "negated-enabled/tsconfig.json",
        "number-disabled/tsconfig.json",
        "relational-disabled/tsconfig.json",
        "string-disabled/tsconfig.json",
        "number-enabled/tsconfig.json",
        "relational-enabled/tsconfig.json",
        "string-enabled/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "negated-enabled/tsconfig.json".to_string(),
            "number-enabled/tsconfig.json".to_string(),
            "relational-enabled/tsconfig.json".to_string(),
            "string-enabled/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn reusable_conditions_resolve_static_string_functions_across_call_inputs() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with: {label: Release-Candidate}\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, required: true}\njobs:\n  contains-true:\n    if: contains(inputs.label, 'LEASE-CAN')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project contains-true/tsconfig.json\n  starts-true:\n    if: startsWith(inputs.label, 'release')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project starts-true/tsconfig.json\n  ends-true:\n    if: endsWith(inputs.label, 'CANDIDATE')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ends-true/tsconfig.json\n  contains-false:\n    if: contains(inputs.label, 'nightly')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project contains-false/tsconfig.json\n  starts-false:\n    if: startsWith(inputs.label, 'candidate')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project starts-false/tsconfig.json\n  ends-false:\n    if: endsWith(inputs.label, '.ts')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ends-false/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "contains-true/tsconfig.json",
        "starts-true/tsconfig.json",
        "ends-true/tsconfig.json",
        "contains-false/tsconfig.json",
        "starts-false/tsconfig.json",
        "ends-false/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "contains-true/tsconfig.json".to_string(),
            "ends-true/tsconfig.json".to_string(),
            "starts-true/tsconfig.json".to_string(),
        ])
    );
}
