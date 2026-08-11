use super::*;

#[test]
fn scanner_does_not_credit_typechecks_behind_literal_false_format_comparisons() {
    let parsed = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  format-disabled:\n    if: \"${{ format('checks-{0}', 'main') == 'checks-release' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project format-disabled/tsconfig.json\n  escaped-format-disabled:\n    if: \"${{ format('{{{0}}}', 'main') == '{release}' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project escaped-format-disabled/tsconfig.json\n  negative-zero-disabled:\n    if: \"${{ format('{0}', -0) == '0' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project negative-zero-disabled/tsconfig.json\n  dynamic-format:\n    if: \"${{ format('checks-{0}', github.ref) == 'checks-main' }}\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic-format/tsconfig.json\n",
        )],
    };
    let tracked = [
        "dynamic-format/tsconfig.json",
        "escaped-format-disabled/tsconfig.json",
        "format-disabled/tsconfig.json",
        "negative-zero-disabled/tsconfig.json",
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
        BTreeSet::from(["dynamic-format/tsconfig.json".to_string()])
    );
}
