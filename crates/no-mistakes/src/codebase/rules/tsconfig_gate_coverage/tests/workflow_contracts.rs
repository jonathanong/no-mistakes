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
        workflow(
            ".github/workflows/expression-ref.yml",
            "on: push\njobs:\n  invalid-call:\n    uses: owner/repo/.github/workflows/checks.yml@${{ github.sha }}\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project expression-ref/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/static-invalid-ref.yml",
            "on: push\njobs:\n  invalid-call:\n    uses: owner/repo/.github/workflows/checks.yml@main branch\n  sibling:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project static-invalid-ref/tsconfig.json\n",
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
                "valid-remote",
                "expression-ref",
                "static-invalid-ref"
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
fn unavailable_contexts_in_reusable_call_inputs_earn_no_credit() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n    with:\n      token: '${{ secrets.TOKEN }}'\n",
        ),
        workflow(
            ".github/workflows/callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      token:\n        type: string\n        required: true\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unavailable-context/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/direct.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(documents, &["unavailable-context", "direct"]),
        BTreeSet::from(["direct/tsconfig.json".to_string()])
    );
}

#[test]
fn literal_from_json_collections_cannot_activate_scalar_reusable_workflows() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n    with:\n      enabled: '${{ fromJSON(''[]'') }}'\n",
        ),
        workflow(
            ".github/workflows/callee.yml",
            "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project collection-binding/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/direct.yml",
            "on: push\njobs:\n  direct:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(documents, &["collection-binding", "direct"]),
        BTreeSet::from(["direct/tsconfig.json".to_string()])
    );
}

#[test]
fn literal_from_json_array_conditions_skip_unreachable_typechecks() {
    let documents = vec![workflow(
        ".github/workflows/conditional.yml",
        "on: push\njobs:\n  skipped:\n    if: contains(fromJSON('[\"schedule\"]'), github.event_name)\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project skipped/tsconfig.json\n  retained:\n    if: contains(fromJSON('[\"push\", \"schedule\"]'), github.event_name)\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project retained/tsconfig.json\n",
    )];

    assert_eq!(
        scanned(documents, &["skipped", "retained"]),
        BTreeSet::from(["retained/tsconfig.json".to_string()])
    );
}

#[test]
fn literal_join_conditions_skip_unreachable_typechecks() {
    let documents = vec![workflow(
        ".github/workflows/conditional.yml",
        "on: push\njobs:\n  skipped:\n    if: join(fromJSON('[\"release\"]'), ',') == 'candidate'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project skipped/tsconfig.json\n  invalid:\n    if: join(fromJSON('[]'), fromJSON('not-json')) == ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid/tsconfig.json\n  retained:\n    if: join(fromJSON('[\"push\", \"schedule\"]'), '-') == 'push-schedule'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project retained/tsconfig.json\n",
    )];

    assert_eq!(
        scanned(documents, &["skipped", "invalid", "retained"]),
        BTreeSet::from(["retained/tsconfig.json".to_string()])
    );
}

#[test]
fn literal_from_json_array_comparisons_preserve_distinct_instance_semantics() {
    let documents = vec![workflow(
        ".github/workflows/conditional.yml",
        "on: push\njobs:\n  equal:\n    if: fromJSON('[]') == fromJSON('[]')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project equal/tsconfig.json\n  unequal:\n    if: fromJSON('[]') != fromJSON('[]')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unequal/tsconfig.json\n",
    )];

    assert_eq!(
        scanned(documents, &["equal", "unequal"]),
        BTreeSet::from(["unequal/tsconfig.json".to_string()])
    );
}

#[test]
fn literal_from_json_objects_in_arrays_do_not_credit_typechecks() {
    let documents = vec![workflow(
        ".github/workflows/conditional.yml",
        "on: push\njobs:\n  skipped:\n    if: contains(fromJSON('[{}]'), 'x')\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project skipped/tsconfig.json\n  retained:\n    if: contains(fromJSON('[{}, \"push\"]'), github.event_name)\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project retained/tsconfig.json\n",
    )];

    assert_eq!(
        scanned(documents, &["skipped", "retained"]),
        BTreeSet::from(["retained/tsconfig.json".to_string()])
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
            "on: push\njobs:\n  empty-call:\n    strategy:\n      matrix:\n        target: []\n    uses: ./.github/workflows/callee.yml\n  internal-call:\n    uses: ./.github/workflows/internal.yml\n",
        ),
        workflow(
            ".github/workflows/include-only.yml",
            "on: push\njobs:\n  include-restores:\n    strategy:\n      matrix:\n        include:\n          - target: linux\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project include-restores/tsconfig.json\n  dynamic-include:\n    strategy:\n      matrix:\n        include: '${{ fromJSON(needs.setup.outputs.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic-include/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid.yml",
            "on: push\njobs:\n  valid:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid/tsconfig.json\n",
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

#[test]
fn workflow_call_input_default_expressions_are_validated_and_preserved() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/defaults.yml\n",
        ),
        workflow(
            ".github/workflows/defaults.yml",
            "on:\n  workflow_call:\n    inputs:\n      disabled:\n        type: boolean\n        default: '${{ false }}'\n      enabled:\n        type: boolean\n        default: '${{ true }}'\n      compared:\n        type: boolean\n        default: '${{ true == false }}'\n      logical:\n        type: boolean\n        default: '${{ true && false }}'\n      contained:\n        type: boolean\n        default: \"${{ contains('x', 'y') }}\"\n      attempts:\n        type: number\n        default: '${{ 0 }}'\n      label:\n        type: string\n        default: 'release-${{ github.ref_name }}'\njobs:\n  disabled:\n    if: inputs.disabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project disabled-default/tsconfig.json\n  enabled:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project enabled-default/tsconfig.json\n  compared:\n    if: inputs.compared\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project compared-default/tsconfig.json\n  logical:\n    if: inputs.logical\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project logical-default/tsconfig.json\n  contained:\n    if: inputs.contained\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project contained-default/tsconfig.json\n  zero:\n    if: inputs.attempts\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project zero-default/tsconfig.json\n  dynamic:\n    if: inputs.label\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic-default/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/json-caller.yml",
            "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/json-defaults.yml\n",
        ),
        workflow(
            ".github/workflows/json-defaults.yml",
            "on:\n  workflow_call:\n    inputs:\n      disabled:\n        type: boolean\n        default: \"${{ fromJSON('false') }}\"\n      attempts:\n        type: number\n        default: \"${{ fromJSON('0') }}\"\njobs:\n  disabled:\n    if: inputs.disabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project json-disabled-default/tsconfig.json\n  zero:\n    if: inputs.attempts\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project json-zero-default/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/malformed-default.yml",
            "on:\n  push:\n  workflow_call:\n    inputs:\n      label:\n        type: string\n        default: '${{ }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project malformed-default/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unavailable-default.yml",
            "on:\n  push:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\n        default: '${{ secrets.TOKEN }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project unavailable-default/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/mismatched-default.yml",
            "on:\n  push:\n  workflow_call:\n    inputs:\n      label:\n        type: string\n        default: '${{ true }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project mismatched-default/tsconfig.json\n",
        ),
    ];

    assert_eq!(
        scanned(
            documents,
            &[
                "disabled-default",
                "enabled-default",
                "compared-default",
                "logical-default",
                "contained-default",
                "json-disabled-default",
                "json-zero-default",
                "zero-default",
                "dynamic-default",
                "malformed-default",
                "unavailable-default",
                "mismatched-default",
            ],
        ),
        BTreeSet::from([
            "dynamic-default/tsconfig.json".to_string(),
            "enabled-default/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn known_from_json_parse_errors_do_not_credit_reusable_conditions() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/checks.yml\n    with: {payload: not-json}\n",
        ),
        workflow(
            ".github/workflows/checks.yml",
            "on:\n  workflow_call:\n    inputs:\n      payload: {type: string, required: true}\njobs:\n  typecheck:\n    if: fromJSON(inputs.payload) != true\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-json/tsconfig.json\n",
        ),
    ];

    assert!(scanned(documents, &["invalid-json"]).is_empty());
}
