use super::test_support::collect_ci_projects_with_stats;
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

#[test]
fn static_matrix_instances_materialize_strategy_context_values() {
    let parsed = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/typecheck.yml",
            "on: push\njobs:\n  configured:\n    strategy:\n      fail-fast: false\n      max-parallel: 1\n      matrix: {target: [one, two]}\n    runs-on: ubuntu-latest\n    steps:\n      - if: strategy.job-index == 0 && strategy.job-total == 2 && !strategy.fail-fast && strategy.max-parallel == 1\n        run: tsc --noEmit --project configured/tsconfig.json\n  defaults:\n    strategy:\n      matrix: {target: [one, two]}\n    runs-on: ubuntu-latest\n    steps:\n      - if: strategy.fail-fast\n        run: tsc --noEmit --project defaults/tsconfig.json\n  dynamic-configuration:\n    strategy:\n      fail-fast: false\n      max-parallel: 1\n      matrix: '${{ fromJSON(github.event.matrix) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - if: '!strategy.fail-fast && strategy.max-parallel == 1'\n        run: tsc --noEmit --project dynamic-configuration/tsconfig.json\n      - if: 'strategy.fail-fast || strategy.max-parallel != 1'\n        run: tsc --noEmit --project dynamic-impossible/tsconfig.json\n  impossible-instance:\n    strategy:\n      matrix: {target: [one, two]}\n    runs-on: ubuntu-latest\n    steps:\n      - if: strategy.job-index == 2\n        run: tsc --noEmit --project impossible/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "configured/tsconfig.json".to_string(),
        "defaults/tsconfig.json".to_string(),
        "dynamic-configuration/tsconfig.json".to_string(),
        "dynamic-impossible/tsconfig.json".to_string(),
        "impossible/tsconfig.json".to_string(),
    ]);
    let inputs = ProjectSourceInputs::from([
        (
            "configured/tsconfig.json".to_string(),
            BTreeSet::from(["configured/src/index.ts".to_string()]),
        ),
        (
            "defaults/tsconfig.json".to_string(),
            BTreeSet::from(["defaults/src/index.ts".to_string()]),
        ),
        (
            "dynamic-configuration/tsconfig.json".to_string(),
            BTreeSet::from(["dynamic-configuration/src/index.ts".to_string()]),
        ),
        (
            "dynamic-impossible/tsconfig.json".to_string(),
            BTreeSet::from(["dynamic-impossible/src/index.ts".to_string()]),
        ),
        (
            "impossible/tsconfig.json".to_string(),
            BTreeSet::from(["impossible/src/index.ts".to_string()]),
        ),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &inputs).0,
        BTreeSet::from([
            "configured/tsconfig.json".to_string(),
            "defaults/tsconfig.json".to_string(),
            "dynamic-configuration/tsconfig.json".to_string(),
        ])
    );
}
