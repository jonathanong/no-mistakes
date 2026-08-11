use super::*;

#[test]
fn exact_ref_names_are_type_checked_in_resolved_job_fields() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on:\n  push:\n    branches: [main]\njobs:\n  invalid-concurrency:\n    concurrency:\n      group: checks\n      cancel-in-progress: '${{ github.ref_name || false }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p concurrency/tsconfig.json\n  invalid-url:\n    environment:\n      name: production\n      url: '${{ case(true, fromJSON(github.ref_name), ''ok'') }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p environment/tsconfig.json\n  unknown-config:\n    if: vars.ENABLED\n    concurrency:\n      group: checks\n      cancel-in-progress: '${{ github.ref_name }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  dependent:\n    needs: unknown-config\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "concurrency/tsconfig.json".to_string(),
        "dependent/tsconfig.json".to_string(),
        "environment/tsconfig.json".to_string(),
    ]);

    assert!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked))
            .0
            .is_empty()
    );
}

#[test]
fn invalid_resolved_environment_urls_fail_dependent_jobs() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            document(
                ".github/workflows/caller.yml",
                "on: push\njobs:\n  invalid:\n    uses: ./.github/workflows/invalid.yml\n    with: {url: '{}'}\n  valid:\n    uses: ./.github/workflows/valid.yml\n    with: {url: '\"https://example.test\"'}\n",
            ),
            document(
                ".github/workflows/invalid.yml",
                "on:\n  workflow_call:\n    inputs:\n      url: {type: string, required: true}\njobs:\n  deploy:\n    if: vars.ENABLED\n    environment:\n      name: production\n      url: '${{ fromJSON(inputs.url) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo deploy\n  dependent:\n    needs: deploy\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p invalid/tsconfig.json\n",
            ),
            document(
                ".github/workflows/valid.yml",
                "on:\n  workflow_call:\n    inputs:\n      url: {type: string, required: true}\njobs:\n  deploy:\n    environment:\n      name: production\n      url: '${{ fromJSON(inputs.url) }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo deploy\n  dependent:\n    needs: deploy\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p valid/tsconfig.json\n",
            ),
        ],
    };
    let tracked = BTreeSet::from([
        "invalid/tsconfig.json".to_string(),
        "valid/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}
