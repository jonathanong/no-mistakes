use super::test_support::collect_ci_projects_with_stats;
use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowDocument;

fn document(path: &str, yaml: &str) -> ParsedWorkflowDocument {
    ParsedWorkflowDocument {
        path: path.to_string(),
        value: Ok(serde_yaml::from_str(yaml).unwrap()),
    }
}

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

#[test]
fn reusable_secret_availability_survives_only_explicit_or_known_inheritance() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/absent-root.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/absent-intermediate.yml\n    secrets:\n      other: '${{ secrets.OTHER }}'\n",
            ),
            document(
                ".github/workflows/absent-intermediate.yml",
                "on:\n  workflow_call:\n    secrets:\n      other: {required: true}\njobs:\n  call:\n    uses: ./.github/workflows/absent-leaf.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/absent-leaf.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p absent/tsconfig.json\n",
            ),
            document(
                ".github/workflows/explicit-root.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/explicit-intermediate.yml\n    secrets:\n      token: '${{ secrets.TOKEN }}'\n",
            ),
            document(
                ".github/workflows/explicit-intermediate.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  call:\n    uses: ./.github/workflows/explicit-leaf.yml\n    secrets:\n      token: '${{ secrets.token }}'\n",
            ),
            document(
                ".github/workflows/explicit-leaf.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p explicit/tsconfig.json\n",
            ),
            document(
                ".github/workflows/inherited-root.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/inherited-intermediate.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/inherited-intermediate.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  call:\n    uses: ./.github/workflows/inherited-leaf.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/inherited-leaf.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p inherited/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "absent/tsconfig.json".to_string(),
        "explicit/tsconfig.json".to_string(),
        "inherited/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "explicit/tsconfig.json".to_string(),
            "inherited/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn pull_request_calls_do_not_forward_repository_secrets() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: pull_request\njobs:\n  call:\n    uses: ./.github/workflows/pull-request-callee.yml\n    secrets: inherit\n",
            ),
            document(
                ".github/workflows/pull-request-callee.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p app/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn statically_empty_secret_bindings_remain_empty_in_callees() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n    secrets:\n      token: '${{ github.event.action }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    env: {TOKEN: '${{ secrets.token }}'}\n    steps:\n      - if: env.TOKEN != ''\n        run: tsc --noEmit -p app/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn reusable_scanner_accepts_job_shape_secret_contexts_and_rejects_unavailable_ones() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/valid-caller.yml",
                "on: push\njobs:\n  strategy:\n    uses: ./.github/workflows/strategy.yml\n    secrets:\n      token: '${{ strategy.job-index }}'\n  matrix:\n    uses: ./.github/workflows/matrix.yml\n    strategy:\n      matrix: {token: [one]}\n    secrets:\n      token: '${{ matrix.token }}'\n  inputs:\n    uses: ./.github/workflows/inputs.yml\n    secrets:\n      token: '${{ inputs.token }}'\n  vars:\n    uses: ./.github/workflows/vars.yml\n    secrets:\n      token: '${{ vars.TOKEN }}'\n",
            ),
            document(
                ".github/workflows/invalid-caller.yml",
                "on: push\njobs:\n  call:\n    uses: ./.github/workflows/invalid.yml\n    secrets:\n      token: '${{ env.TOKEN }}'\n",
            ),
            document(
                ".github/workflows/strategy.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p strategy/tsconfig.json\n",
            ),
            document(
                ".github/workflows/matrix.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p matrix/tsconfig.json\n",
            ),
            document(
                ".github/workflows/inputs.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p inputs/tsconfig.json\n",
            ),
            document(
                ".github/workflows/vars.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p vars/tsconfig.json\n",
            ),
            document(
                ".github/workflows/invalid.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = ["strategy", "matrix", "inputs", "vars", "invalid"]
        .into_iter()
        .map(|name| format!("{name}/tsconfig.json"))
        .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "inputs/tsconfig.json".to_string(),
            "matrix/tsconfig.json".to_string(),
            "strategy/tsconfig.json".to_string(),
            "vars/tsconfig.json".to_string(),
        ])
    );
}
