use super::*;

fn workflow(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

fn scanned(documents: Vec<ParsedWorkflowDocument>, projects: &[&str]) -> BTreeSet<String> {
    let tracked = projects
        .iter()
        .map(|project| format!("{project}/tsconfig.json"))
        .collect();
    ci_typechecked_projects(
        &ParsedWorkflowSet { documents },
        &tracked,
        &project_inputs(&tracked),
    )
}

#[test]
fn skipped_local_calls_still_require_valid_contracts() {
    let documents = vec![
        workflow(
            ".github/workflows/root.yml",
            "on: push\njobs:\n  skipped:\n    if: false\n    uses: ./.github/workflows/callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project sibling/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      required:\n        type: boolean\n        required: true\njobs: {}\n",
        ),
        workflow(
            ".github/workflows/noncanonical.yml",
            "on: push\njobs:\n  invalid-call:\n    uses: ./.github/workflows/subdir/../callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project noncanonical/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/backslash.yml",
            "on: push\njobs:\n  invalid-call:\n    uses: './.github/workflows/subdir\\..\\callee.yml'\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project backslash/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-remote.yml",
            "on: push\njobs:\n  invalid-call:\n    uses: owner/repo/.github/workflows/checks.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-remote/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-remote.yml",
            "on: push\njobs:\n  opaque-call:\n    uses: owner/repo/.github/workflows/checks.yml@main\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-remote/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/empty-ref.yml",
            "on: push\njobs:\n  invalid-call:\n    uses: owner/repo/.github/workflows/checks.yml@\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-ref/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(
            documents,
            &[
                "sibling",
                "noncanonical",
                "backslash",
                "invalid-remote",
                "empty-ref",
                "valid-remote"
            ]
        ),
        BTreeSet::from(["valid-remote/tsconfig.json".to_string()])
    );
}

#[test]
fn direct_activations_reject_invalid_or_falsy_workflow_call_input_declarations() {
    let documents = vec![
        workflow(
            ".github/workflows/missing-type.yml",
            "on:\n  push:\n  workflow_call:\n    inputs:\n      enabled:\n        default: true\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project missing-type/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/wrong-default.yml",
            "on:\n  push:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        default: text\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project wrong-default/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-string.yml",
            "on:\n  push:\n  workflow_call:\n    inputs:\n      label:\n        type: string\njobs:\n  typecheck:\n    if: inputs.label\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-string/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(
            documents,
            &["missing-type", "wrong-default", "valid-string"]
        ),
        BTreeSet::new()
    );
}

#[test]
fn undeclared_inputs_are_false_while_declared_nonbooleans_are_unknown() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      label: release\n",
        ),
        workflow(
            ".github/workflows/callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      label:\n        type: string\njobs:\n  missing:\n    if: inputs.missing\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project missing/tsconfig.json\n  negated:\n    if: '!inputs.missing'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project negated/tsconfig.json\n  equal-false:\n    if: inputs.missing == false\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project equal-false/tsconfig.json\n  unequal-false:\n    if: inputs.missing != false\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unequal-false/tsconfig.json\n  declared-string:\n    if: inputs.label\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project declared-string/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(
            documents,
            &[
                "missing",
                "negated",
                "equal-false",
                "unequal-false",
                "declared-string"
            ]
        ),
        BTreeSet::from([
            "declared-string/tsconfig.json".to_string(),
            "equal-false/tsconfig.json".to_string(),
            "negated/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn compact_boolean_bindings_and_literal_first_comparisons_are_static() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      disabled: '${{false}}'\n      enabled: '${{true}}'\n",
        ),
        workflow(
            ".github/workflows/callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      disabled:\n        type: boolean\n        required: true\n      enabled:\n        type: boolean\n        required: true\njobs:\n  compact-false:\n    if: inputs.disabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project compact-false/tsconfig.json\n  reversed-false:\n    if: false == inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project reversed-false/tsconfig.json\n  reversed-true:\n    if: true == inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project reversed-true/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(
            documents,
            &["compact-false", "reversed-false", "reversed-true"]
        ),
        BTreeSet::from(["reversed-true/tsconfig.json".to_string()])
    );
}

#[test]
fn malformed_contract_containers_and_empty_matrices_earn_no_credit() {
    let documents = vec![
        workflow(
            ".github/workflows/malformed-direct.yml",
            "on:\n  push:\n  workflow_call:\n    inputs: [invalid]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-direct/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-empty-contract.yml",
            "on:\n  push:\n  workflow_call:\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-empty-contract/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/malformed-callee.yml\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-local/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-callee.yml",
            "on:\n  workflow_call:\n    input: {}\njobs: {}\n",
        ),
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  empty-call:\n    strategy:\n      matrix:\n        target: []\n    uses: ./.github/workflows/callee.yml\n  internal-call:\n    uses: ./.github/workflows/internal.yml\n  include-restores:\n    strategy:\n      matrix:\n        target: []\n        include:\n          - target: linux\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project include-restores/tsconfig.json\n  dynamic-include:\n    strategy:\n      matrix:\n        target: []\n        include: '${{ fromJSON(needs.setup.outputs.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic-include/tsconfig.json\n  valid:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/callee.yml",
            "on: workflow_call\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-call/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/internal.yml",
            "on: workflow_call\njobs:\n  empty-job:\n    strategy:\n      matrix:\n        target: []\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty-job/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(
            documents,
            &[
                "malformed-direct",
                "malformed-local",
                "empty-call",
                "empty-job",
                "valid-empty-contract",
                "include-restores",
                "dynamic-include",
                "valid"
            ]
        ),
        BTreeSet::from([
            "dynamic-include/tsconfig.json".to_string(),
            "include-restores/tsconfig.json".to_string(),
            "valid-empty-contract/tsconfig.json".to_string(),
            "valid/tsconfig.json".to_string(),
        ])
    );
}
