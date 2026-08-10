use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn workflow_document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
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
fn invalid_non_reusable_trigger_configuration_earns_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on:\n  push:\n    paths: ['src/**']\n    paths-ignore: ['src/generated/**']\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-trigger/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from(["invalid-trigger/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn invalid_activity_type_and_cron_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/issues.yml",
                "on:\n  issues:\n    types: [not_an_issue_event]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-type/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/schedule.yml",
                "on:\n  schedule:\n    - cron: nope\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-cron/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-type/tsconfig.json".to_string(),
        "invalid-cron/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn unconfigured_or_expression_trigger_earns_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/schedule.yml",
                "on: schedule\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project scalar-schedule/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/push.yml",
                "on:\n  push:\n    paths: ['${{ github.event.repository.name }}/**']\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project expression-path/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "scalar-schedule/tsconfig.json".to_string(),
        "expression-path/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn zero_instance_matrix_needs_skip_dependents_unless_they_continue_explicitly() {
    let workflows = ParsedWorkflowSet {
        documents: vec![workflow_document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  setup:\n    strategy:\n      matrix:\n        target: [ubuntu-latest]\n        exclude:\n          - target: ubuntu-latest\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo skipped\n  blocked:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project blocked/tsconfig.json\n  implicit-success:\n    needs: setup\n    if: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project implicit/tsconfig.json\n  continues:\n    needs: setup\n    if: '${{ always() }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project continues/tsconfig.json\n  compound-continues:\n    needs: setup\n    if: '${{ always() && true }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project compound/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "blocked/tsconfig.json".to_string(),
        "implicit/tsconfig.json".to_string(),
        "continues/tsconfig.json".to_string(),
        "compound/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "compound/tsconfig.json".to_string(),
            "continues/tsconfig.json".to_string(),
        ])
    );
}
