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

#[test]
fn reusable_inputs_preserve_scalar_comparisons_across_defaults_and_forwarding() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  forwarded:\n    uses: ./.github/workflows/intermediate.yml\n    with: {label: Release, count: 2}\n  defaults:\n    uses: ./.github/workflows/defaults.yml\n  interpolated:\n    uses: ./.github/workflows/interpolated.yml\n    with:\n      label: 'release-${{ github.ref_name }}'\n",
            ),
            document(
                ".github/workflows/intermediate.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, required: true}\n      count: {type: number, required: true}\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      label: '${{ inputs.label }}'\n      count: '${{ inputs.count }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, required: true}\n      count: {type: number, required: true}\njobs:\n  matching:\n    if: inputs.label == 'release' && inputs.count == 2\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project forwarded-match/tsconfig.json\n  mismatching:\n    if: inputs.label != 'RELEASE' || inputs.count != 2\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project forwarded-mismatch/tsconfig.json\n",
            ),
            document(
                ".github/workflows/defaults.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, default: release}\n      count: {type: number, default: 2}\njobs:\n  matching:\n    if: inputs.label == 'RELEASE' && inputs.count == 2\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project default-match/tsconfig.json\n  mismatching:\n    if: inputs.label != 'release' || inputs.count != 2\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project default-mismatch/tsconfig.json\n",
            ),
            document(
                ".github/workflows/interpolated.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, required: true}\njobs:\n  dynamic:\n    if: inputs.label == 'release-main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project interpolated/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "default-match/tsconfig.json",
        "default-mismatch/tsconfig.json",
        "forwarded-match/tsconfig.json",
        "forwarded-mismatch/tsconfig.json",
        "interpolated/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs).0,
        BTreeSet::from([
            "default-match/tsconfig.json".to_string(),
            "forwarded-match/tsconfig.json".to_string(),
            "interpolated/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn incompatible_exact_input_forwarding_invalidates_the_reusable_path() {
    let parsed = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/intermediate.yml\n    with: {label: release}\n",
            ),
            document(
                ".github/workflows/intermediate.yml",
                "on:\n  workflow_call:\n    inputs:\n      label: {type: string, required: true}\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      count: '${{ inputs.label }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      count: {type: number, required: true}\njobs:\n  typecheck:\n    if: inputs.count != 0\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-forward/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["invalid-forward/tsconfig.json".to_string()]);
    let project_inputs = tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect();

    assert!(
        collect_ci_projects_with_stats(&parsed, &tracked, &project_inputs)
            .0
            .is_empty()
    );
}
