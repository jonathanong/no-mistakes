use super::*;

#[test]
fn ci_scanner_rejects_push_callers_with_invalid_workflow_dispatch_choices() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                "invalid-choice.yml",
                "on:\n  push:\n  workflow_dispatch:\n    inputs:\n      target:\n        type: choice\n        options: [staging, '']\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-choice/tsconfig.json\n",
            ),
            workflow_document(
                "invalid-default.yml",
                "on:\n  push:\n  workflow_dispatch:\n    inputs:\n      target:\n        type: choice\n        options: [staging, production]\n        default: true\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-default/tsconfig.json\n",
            ),
            workflow_document(
                "valid-controls.yml",
                "on:\n  push:\n  workflow_dispatch:\n    inputs:\n      target:\n        type: choice\n        options: [staging, production]\n        default: production\n      enabled:\n        type: boolean\n        default: false\n      retries:\n        type: number\n        default: 2\n      label:\n        type: string\n        default: release\n      environment:\n        type: environment\n        default: production\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-controls/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "invalid-choice/tsconfig.json",
        "invalid-default/tsconfig.json",
        "valid-controls/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from(["valid-controls/tsconfig.json".to_string()])
    );
}
