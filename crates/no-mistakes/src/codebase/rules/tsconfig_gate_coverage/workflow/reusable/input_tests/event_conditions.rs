use super::*;

#[test]
fn reusable_scanner_resolves_bracketed_event_name_bindings_for_push_and_schedule() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on:\n  push:\n  schedule:\n    - cron: '0 0 * * *'\njobs:\n  valid:\n    uses: ./.github/workflows/callee.yml\n    with:\n      enabled: \"${{ GiThUb [ 'EVENT_NAME' ] == 'push' }}\"\n",
            ),
            document(
                ".github/workflows/malformed-caller.yml",
                "on: push\njobs:\n  malformed:\n    uses: ./.github/workflows/malformed.yml\n    with:\n      enabled: \"${{ github['event_name' == 'push' }}\"\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project push/tsconfig.json\n",
            ),
            document(
                ".github/workflows/malformed.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "malformed/tsconfig.json".to_string(),
        "push/tsconfig.json".to_string(),
    ]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from(["push/tsconfig.json".to_string()])
    );
}
