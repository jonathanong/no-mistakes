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
fn only_success_path_valid_context_linux_and_source_change_gates_count() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/status.yml",
                "on: push\njobs:\n  failure:\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure/tsconfig.json\n  cancelled:\n    runs-on: ubuntu-latest\n    steps:\n      - if: cancelled()\n        run: tsc --noEmit -p cancelled/tsconfig.json\n  success:\n    if: success()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p success/tsconfig.json\n",
            ),
            document(
                ".github/workflows/invalid-job-context.yml",
                "on: push\njobs:\n  typecheck:\n    if: secrets.TYPECHECK\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-job-context/tsconfig.json\n",
            ),
            document(
                ".github/workflows/invalid-step-context.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - if: secrets.TYPECHECK\n        run: tsc --noEmit -p invalid-step-context/tsconfig.json\n",
            ),
            document(
                ".github/workflows/invalid-job-function.yml",
                "on: push\njobs:\n  typecheck:\n    if: hashFiles('**/pnpm-lock.yaml') != ''\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-job-function/tsconfig.json\n",
            ),
            document(
                ".github/workflows/containers.yml",
                "on: push\njobs:\n  linux:\n    runs-on: ubuntu-latest\n    container: node:22\n    steps:\n      - run: tsc --noEmit -p linux-container/tsconfig.json\n",
            ),
            document(
                ".github/workflows/windows-container.yml",
                "on: push\njobs:\n  invalid:\n    runs-on: windows-latest\n    container: node:22\n    steps:\n      - shell: bash\n        run: tsc --noEmit -p windows-container/tsconfig.json\n",
            ),
            document(
                ".github/workflows/custom-container.yml",
                "on: push\njobs:\n  unknown:\n    runs-on: custom-runner\n    container: node:22\n    steps:\n      - run: tsc --noEmit -p custom-container/tsconfig.json\n",
            ),
            document(
                ".github/workflows/closed.yml",
                "on:\n  pull_request:\n    types: [closed]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p closed/tsconfig.json\n",
            ),
            document(
                ".github/workflows/synchronize.yml",
                "on:\n  pull_request:\n    types: [synchronize]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p synchronize/tsconfig.json\n",
            ),
            document(
                ".github/workflows/push-fallback.yml",
                "on:\n  push:\n  pull_request:\n    types: [closed]\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p push-fallback/tsconfig.json\n",
            ),
        ],
    };
    let tracked = [
        "failure",
        "cancelled",
        "success",
        "invalid-job-context",
        "invalid-step-context",
        "invalid-job-function",
        "linux-container",
        "windows-container",
        "custom-container",
        "closed",
        "synchronize",
        "push-fallback",
    ]
    .into_iter()
    .map(|name| format!("{name}/tsconfig.json"))
    .collect();

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "linux-container/tsconfig.json".to_string(),
            "push-fallback/tsconfig.json".to_string(),
            "success/tsconfig.json".to_string(),
            "synchronize/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn event_sensitive_inputs_stay_correlated_with_each_events_path_filters() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on:\n  push:\n    paths: ['other/**']\n  pull_request:\n    paths: ['app/**']\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    with:\n      enabled: \"${{ github.event_name == 'push' }}\"\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    inputs:\n      enabled: {type: boolean, required: true}\njobs:\n  typecheck:\n    if: inputs.enabled\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p app/tsconfig.json\n      - run: tsc --noEmit -p other/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "app/tsconfig.json".to_string(),
        "other/tsconfig.json".to_string(),
    ]);
    let inputs = ProjectSourceInputs::from([
        (
            "app/tsconfig.json".to_string(),
            BTreeSet::from(["app/src/index.ts".to_string()]),
        ),
        (
            "other/tsconfig.json".to_string(),
            BTreeSet::from(["other/src/index.ts".to_string()]),
        ),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &inputs).0,
        BTreeSet::from(["other/tsconfig.json".to_string()])
    );
}

#[test]
fn unavailable_reusable_secret_expressions_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  checks:\n    uses: ./.github/workflows/callee.yml\n    secrets:\n      token: '${{ success() }}'\n",
            ),
            document(
                ".github/workflows/callee.yml",
                "on:\n  workflow_call:\n    secrets:\n      token: {required: true}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-secret/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from(["invalid-secret/tsconfig.json".to_string()]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn invalid_continue_on_error_and_environment_contexts_earn_no_coverage() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/job-continue.yml",
                "on: push\njobs:\n  typecheck:\n    continue-on-error: '${{ failure() }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-job-continue/tsconfig.json\n",
            ),
            document(
                ".github/workflows/step-continue.yml",
                "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - continue-on-error: '${{ failure() }}'\n        run: tsc --noEmit -p invalid-step-continue/tsconfig.json\n",
            ),
            document(
                ".github/workflows/environment.yml",
                "on: push\njobs:\n  typecheck:\n    environment: '${{ secrets.DEPLOY_ENV }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid-environment/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on: push\njobs:\n  typecheck:\n    environment:\n      name: '${{ matrix.deployment }}'\n      url: '${{ steps.deploy.outputs.url }}'\n    strategy:\n      matrix:\n        deployment: [staging]\n    runs-on: ubuntu-latest\n    steps:\n      - id: deploy\n        run: tsc --noEmit -p valid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid-job-continue/tsconfig.json".to_string(),
        "invalid-step-continue/tsconfig.json".to_string(),
        "invalid-environment/tsconfig.json".to_string(),
        "valid/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}
