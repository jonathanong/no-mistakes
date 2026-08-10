use super::*;

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

fn workflow_document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

#[test]
fn ci_scanner_treats_omitted_or_falsy_nonboolean_inputs_as_false() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/checks.yml\n    with:\n      bound-zero: 0\n      bound-empty: ''\n      bound-text: release\n      bound-number: 2\n      dynamic: '${{ needs.detect.outputs.value }}'\n",
            ),
            workflow_document(
                ".github/workflows/checks.yml",
                "on:\n  workflow_call:\n    inputs:\n      omitted-string:\n        type: string\n      zero:\n        type: number\n        default: 0\n      empty:\n        type: string\n        default: ''\n      truthy:\n        type: string\n        default: fallback\n      bound-zero:\n        type: number\n      bound-empty:\n        type: string\n      bound-text:\n        type: string\n      bound-number:\n        type: number\n      dynamic:\n        type: string\njobs:\n  omitted:\n    if: inputs.omitted-string\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project omitted/tsconfig.json\n  zero:\n    if: inputs.zero\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project zero/tsconfig.json\n  empty:\n    if: inputs.empty\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty/tsconfig.json\n  truthy:\n    if: inputs.truthy\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project truthy/tsconfig.json\n  truthy-negated:\n    if: '!inputs.truthy'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project truthy-negated/tsconfig.json\n  bound-zero:\n    if: inputs.bound-zero\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bound-zero/tsconfig.json\n  bound-empty:\n    if: inputs.bound-empty\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bound-empty/tsconfig.json\n  bound-text-negated:\n    if: '!inputs.bound-text'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bound-text-negated/tsconfig.json\n  bound-number-negated:\n    if: '!inputs.bound-number'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bound-number-negated/tsconfig.json\n  dynamic:\n    if: inputs.dynamic\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/direct.yml",
                "on:\n  push:\n  workflow_call:\n    inputs:\n      omitted:\n        type: string\n        default: fallback\njobs:\n  typecheck:\n    if: inputs.omitted\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project direct-omitted/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "direct-omitted/tsconfig.json",
        "bound-empty/tsconfig.json",
        "bound-number-negated/tsconfig.json",
        "bound-text-negated/tsconfig.json",
        "bound-zero/tsconfig.json",
        "dynamic/tsconfig.json",
        "empty/tsconfig.json",
        "omitted/tsconfig.json",
        "truthy/tsconfig.json",
        "truthy-negated/tsconfig.json",
        "zero/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "dynamic/tsconfig.json".to_string(),
            "truthy/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn ci_scanner_validates_contract_identifiers_outputs_and_bracket_input_references() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            workflow_document(
                ".github/workflows/bracket-caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/bracket.yml\n    with:\n      enabled: false\n",
            ),
            workflow_document(
                ".github/workflows/bracket-malformed-caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/bracket-malformed.yml\n    with:\n      enabled: false\n",
            ),
            workflow_document(
                ".github/workflows/bracket.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\njobs:\n  direct:\n    if: inputs['enabled']\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bracket-direct/tsconfig.json\n  negated:\n    if: \"! inputs [ 'enabled' ]\"\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bracket-negated/tsconfig.json\n  compared:\n    if: inputs['enabled'] == false\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bracket-compared/tsconfig.json\n  logical:\n    # Do not mistake a compound expression for one input reference.\n    if: inputs.enabled || github.event_name == 'push'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project logical/tsconfig.json\n  short-circuited:\n    if: inputs.enabled && github.event_name == 'push'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project short-circuited/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/bracket-malformed.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled:\n        type: boolean\njobs:\n  malformed:\n    if: inputs[enabled]\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project bracket-malformed/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/invalid-input.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/invalid-input-callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/invalid-input-callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      invalid.name:\n        type: boolean\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-input/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/invalid-secret.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/invalid-secret-callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/invalid-secret-callee.yml",
                "on:\n  workflow_call:\n    secrets:\n      invalid.name:\n        required: false\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-secret/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/output-collision.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/output-collision-callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/output-collision-callee.yml",
                "on:\n  workflow_call:\n    outputs:\n      Result:\n        value: '${{ jobs.check.outputs.value }}'\n      result:\n        value: '${{ jobs.check.outputs.value }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project output-collision/tsconfig.json\n",
            ),
            workflow_document(
                ".github/workflows/invalid-output.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/invalid-output-callee.yml\n",
            ),
            workflow_document(
                ".github/workflows/invalid-output-callee.yml",
                "on:\n  workflow_call:\n    outputs:\n      invalid.name:\n        value: '${{ jobs.check.outputs.value }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-output/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "bracket-compared/tsconfig.json",
        "bracket-direct/tsconfig.json",
        "bracket-malformed/tsconfig.json",
        "bracket-negated/tsconfig.json",
        "invalid-input/tsconfig.json",
        "invalid-secret/tsconfig.json",
        "invalid-output/tsconfig.json",
        "logical/tsconfig.json",
        "output-collision/tsconfig.json",
        "short-circuited/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)),
        BTreeSet::from([
            "bracket-compared/tsconfig.json".to_string(),
            "bracket-negated/tsconfig.json".to_string(),
            "logical/tsconfig.json".to_string(),
        ])
    );
}
