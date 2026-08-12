use super::*;

#[test]
fn static_ordinary_job_outputs_flow_to_needs() {
    let workflow = document(
        ".github/workflows/ordinary-output.yml",
        "on: push\njobs:\n  setup:\n    outputs: {enabled: '${{ true }}', dynamic: '${{ steps.output.outputs.value }}'}\n    runs-on: ubuntu-latest\n    steps:\n      - id: output\n        run: echo setup\n  disabled-by-output:\n    needs: setup\n    if: needs.setup.outputs.enabled == 'false'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p disabled/tsconfig.json\n  dynamic:\n    needs: setup\n    if: needs.setup.outputs.dynamic == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dynamic/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["disabled", "dynamic"]),
        BTreeSet::from(["dynamic/tsconfig.json".to_string()])
    );
}

#[test]
fn reusable_workflow_outputs_flow_to_callers_needs_context() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  call-false:\n    uses: ./.github/workflows/boolean.yml\n    with: {enabled: false}\n  blocked:\n    needs: call-false\n    if: needs.call-false.outputs.enabled == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked/tsconfig.json\n  call-bracket:\n    uses: ./.github/workflows/boolean.yml\n    with: {enabled: true}\n  enabled:\n    needs: call-bracket\n    if: needs['call-bracket']['outputs']['enabled'] == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p enabled/tsconfig.json\n  call-dynamic:\n    uses: ./.github/workflows/dynamic.yml\n  dynamic:\n    needs: call-dynamic\n    if: needs.call-dynamic.outputs.enabled == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dynamic/tsconfig.json\n",
    );
    let boolean_callee = document(
        ".github/workflows/boolean.yml",
        "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\n    outputs:\n      enabled: {value: '${{ inputs.enabled }}'}\njobs:\n  complete:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo complete\n",
    );
    let dynamic_callee = document(
        ".github/workflows/dynamic.yml",
        "on:\n  workflow_call:\n    outputs:\n      enabled: {value: '${{ jobs.complete.outputs.enabled }}'}\njobs:\n  complete:\n    runs-on: ubuntu-latest\n    outputs: {enabled: '${{ steps.output.outputs.enabled }}'}\n    steps:\n      - id: output\n        run: echo complete\n",
    );

    assert_eq!(
        scanned_projects(
            vec![caller, boolean_callee, dynamic_callee],
            &["blocked", "enabled", "dynamic"]
        ),
        BTreeSet::from([
            "dynamic/tsconfig.json".to_string(),
            "enabled/tsconfig.json".to_string()
        ])
    );
}

#[test]
fn reusable_workflow_outputs_resolve_completed_job_outputs() {
    let caller = document(
        ".github/workflows/caller-job-output.yml",
        "on: push\njobs:\n  call:\n    uses: ./.github/workflows/job-output.yml\n  blocked:\n    needs: call\n    if: needs.call.outputs.enabled == 'true'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p blocked/tsconfig.json\n",
    );
    let callee = document(
        ".github/workflows/job-output.yml",
        "on:\n  workflow_call:\n    outputs:\n      enabled: {value: '${{ jobs.complete.outputs.enabled }}'}\njobs:\n  complete:\n    runs-on: ubuntu-latest\n    outputs: {enabled: '${{ false }}'}\n    steps:\n      - run: echo complete\n",
    );

    assert!(scanned_projects(vec![caller, callee], &["blocked"]).is_empty());
}
