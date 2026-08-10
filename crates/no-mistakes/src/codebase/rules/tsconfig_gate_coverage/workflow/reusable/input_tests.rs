use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

#[test]
fn literal_expression_bindings_preserve_truthiness_across_reusable_calls() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      empty: \"${{ ('') }}\"\n      zero: '${{ (0) }}'\n      full: \"${{ ('value') }}\"\n      dynamic: '${{ github.ref }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      empty: {type: string, required: true}\n      zero: {type: number, required: true}\n      full: {type: string, required: true}\n      dynamic: {type: string, required: true}\njobs:\n  empty:\n    if: inputs.empty\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty/tsconfig.json\n  zero:\n    if: inputs.zero\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project zero/tsconfig.json\n  full:\n    if: inputs.full\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project full/tsconfig.json\n  dynamic:\n    if: inputs.dynamic\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic/tsconfig.json\n",
            ),
        ],
    };
    let tracked = ["empty", "zero", "full", "dynamic"]
        .into_iter()
        .map(|name| format!("{name}/tsconfig.json"))
        .collect::<BTreeSet<_>>();
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "dynamic/tsconfig.json".to_string(),
            "full/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn forwarded_nonboolean_inputs_preserve_truthiness_across_reusable_calls() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/intermediate.yml\n    with:\n      empty: ''\n      zero: 0\n      full: release\n      dynamic: '${{ github.ref }}'\n",
            ),
            document(
                ".github/workflows/intermediate.yml",
                "on:\n  workflow_call:\n    inputs:\n      empty: {type: string, required: true}\n      zero: {type: number, required: true}\n      full: {type: string, required: true}\n      dynamic: {type: string, required: true}\njobs:\n  forward:\n    uses: ./.github/workflows/callee.yml\n    with:\n      empty: '${{ inputs.empty }}'\n      zero: '${{ inputs.zero }}'\n      full: '${{ inputs.full }}'\n      dynamic: '${{ inputs.dynamic }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      empty: {type: string, required: true}\n      zero: {type: number, required: true}\n      full: {type: string, required: true}\n      dynamic: {type: string, required: true}\njobs:\n  empty:\n    if: inputs.empty\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project empty/tsconfig.json\n  zero:\n    if: inputs.zero\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project zero/tsconfig.json\n  full:\n    if: inputs.full\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project full/tsconfig.json\n  dynamic:\n    if: inputs.dynamic\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dynamic/tsconfig.json\n",
            ),
        ],
    };
    let tracked = ["empty", "zero", "full", "dynamic"]
        .into_iter()
        .map(|name| format!("{name}/tsconfig.json"))
        .collect::<BTreeSet<_>>();
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "dynamic/tsconfig.json".to_string(),
            "full/tsconfig.json".to_string(),
        ])
    );
}
