use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

#[test]
fn missing_matrix_properties_forward_to_reusable_string_inputs_as_empty() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  no-matrix:\n    uses: ./.github/workflows/callee.yml\n    with: {label: '${{ matrix.missing }}'}\n  missing-axis:\n    strategy:\n      matrix: {enabled: [true]}\n    uses: ./.github/workflows/callee.yml\n    with: {label: '${{ matrix.missing }}'}\n  dynamic:\n    strategy:\n      matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - if: matrix.enabled\n        run: tsc --noEmit --project dynamic/tsconfig.json\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, required: true}\njobs:\n  typecheck:\n    if: inputs.label == ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project app/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "app/tsconfig.json".to_string(),
        "dynamic/tsconfig.json".to_string(),
    ]);
    let inputs = ProjectSourceInputs::from([
        (
            "app/tsconfig.json".to_string(),
            BTreeSet::from(["app/src/index.ts".to_string()]),
        ),
        (
            "dynamic/tsconfig.json".to_string(),
            BTreeSet::from(["dynamic/src/index.ts".to_string()]),
        ),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &inputs).0,
        tracked
    );
}
