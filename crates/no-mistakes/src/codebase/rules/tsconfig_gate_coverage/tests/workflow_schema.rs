use super::*;

fn workflow(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

#[test]
fn invalid_contract_jobs_dependencies_and_empty_matrices_earn_no_credit() {
    let oversized_values = (0..257)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let documents = vec![
        workflow(
            ".github/workflows/declaration-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/declaration-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project declaration/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/declaration-callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        required: 'true'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project declaration/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/jobs-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/jobs-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project jobs/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/jobs-callee.yml",
            "on: workflow_call\njobs: []\n",
        ),
        workflow(
            ".github/workflows/cycle.yml",
            "on: push\njobs:\n  first:\n    needs: second\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project cycle/tsconfig.json\n  second:\n    needs: first\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo blocked\n",
        ),
        workflow(
            ".github/workflows/excluded.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux]\n        exclude:\n          - target: linux\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project excluded/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/partial.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        target: [linux, macos]\n        exclude:\n          - target: linux\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project partial/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/oversized-matrix.yml",
            &format!(
                "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        value: [{oversized_values}]\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project oversized/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project oversized-sibling/tsconfig.json\n"
            ),
        ),
        workflow(
            ".github/workflows/ambiguous-step.yml",
            "on: push\njobs:\n  ambiguous:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ambiguous/tsconfig.json\n        uses: owner/action@v1\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ambiguous-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-remote-bindings.yml",
            "on: push\njobs:\n  remote:\n    uses: owner/repository/.github/workflows/typecheck.yml@main\n    with: true\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project remote-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-strategy.yml",
            "on: push\njobs:\n  malformed:\n    strategy: []\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-strategy/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-strategy-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/misspelled-trigger-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/misspelled-trigger-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/misspelled-trigger-callee.yml",
            "on:\n  workflow_call:\n  workflow_dispath:\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/sequence-misspelled-trigger-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/sequence-misspelled-trigger-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sequence-misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/sequence-misspelled-trigger-callee.yml",
            "on: [workflow_call, workflow_dispath]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sequence-misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-action.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout\n      - run: tsc --noEmit --project malformed-action/tsconfig.json\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-action-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/missing-steps.yml",
            "on: push\njobs:\n  malformed:\n    runs-on: ubuntu-latest\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project missing-steps-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/ordinary-misspelled-trigger-caller.yml",
            "on:\n  push:\n  pussh:\njobs:\n  typecheck:\n    uses: ./.github/workflows/ordinary-misspelled-trigger-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ordinary-misspelled-trigger-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/ordinary-misspelled-trigger-callee.yml",
            "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project ordinary-misspelled-trigger/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/uppercase-inherit.yml",
            "on: push\njobs:\n  remote:\n    uses: owner/repository/.github/workflows/typecheck.yml@main\n    secrets: INHERIT\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project uppercase-inherit-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/empty-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: '${{ }}'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/concatenated-expression-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-expression-callee.yml\n    with:\n      enabled: '${{ true }}${{ false }}'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project concatenated-expression-sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-expression-callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        required: true\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-expression-callee/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "declaration",
        "jobs",
        "cycle",
        "excluded",
        "partial",
        "oversized",
        "oversized-sibling",
        "ambiguous",
        "ambiguous-sibling",
        "remote-sibling",
        "malformed-strategy",
        "malformed-strategy-sibling",
        "misspelled-trigger",
        "sequence-misspelled-trigger",
        "malformed-action",
        "malformed-action-sibling",
        "missing-steps-sibling",
        "ordinary-misspelled-trigger",
        "ordinary-misspelled-trigger-sibling",
        "uppercase-inherit-sibling",
        "empty-expression-sibling",
        "concatenated-expression-sibling",
        "malformed-expression-callee",
    ]
    .into_iter()
    .map(|project| format!("{project}/tsconfig.json"))
    .collect();

    assert_eq!(
        ci_typechecked_projects(
            &ParsedWorkflowSet { documents },
            &tracked,
            &project_inputs(&tracked)
        ),
        BTreeSet::from(["partial/tsconfig.json".to_string()])
    );
}
