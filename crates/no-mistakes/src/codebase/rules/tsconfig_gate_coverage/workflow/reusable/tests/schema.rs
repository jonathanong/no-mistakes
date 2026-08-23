use super::*;

#[test]
fn malformed_workflow_level_fields_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/direct.yml",
                "on: push\ndefaults: []\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/permissions.yml",
                "on: push\npermissions: bogus\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project permissions/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/empty-fields.yml",
                "on: push\nrun-name: ''\nconcurrency: ''\ndefaults:\n  run:\n    shell: ''\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-fields/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/broken-expression.yml",
                "on: push\nrun-name: 'checks-${{ }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project broken-expression/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    outputs:\n      result:\n        value: 'result-${{ }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project callee/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/context-caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/context-callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/context-callee.yml",
                "on:\n  workflow_call:\n    outputs:\n      result:\n        value: '${{ secrets.TOKEN }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project output-context/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "direct/tsconfig.json".to_string(),
        "permissions/tsconfig.json".to_string(),
        "empty-fields/tsconfig.json".to_string(),
        "broken-expression/tsconfig.json".to_string(),
        "callee/tsconfig.json".to_string(),
        "output-context/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn canonical_permission_scopes_earn_coverage_with_supported_access() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/code-quality.yml",
                "on: push\npermissions: {code-quality: write}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project code-quality/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/vulnerability-alerts.yml",
                "on: push\npermissions: {vulnerability-alerts: read}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project vulnerability-alerts/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "code-quality/tsconfig.json".to_string(),
        "vulnerability-alerts/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        tracked
    );
}
