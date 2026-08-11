use super::*;

#[test]
fn known_runner_os_controls_step_conditions() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  linux:\n    runs-on: ubuntu-latest\n    steps:\n      - if: runner.os == 'Windows'\n        run: tsc --noEmit -p windows/tsconfig.json\n      - if: runner.os == 'Linux'\n        run: tsc --noEmit -p linux/tsconfig.json\n  windows:\n    runs-on: windows-latest\n    steps:\n      - shell: bash\n        if: runner.os == 'Linux'\n        run: tsc --noEmit -p wrong-linux/tsconfig.json\n      - shell: bash\n        if: runner.os == 'Windows'\n        run: tsc --noEmit -p known-windows/tsconfig.json\n  grouped:\n    runs-on:\n      group: custom\n      labels: ubuntu-latest\n    steps:\n      - shell: bash\n        if: runner.os == 'Linux'\n        run: tsc --noEmit -p grouped-linux/tsconfig.json\n      - shell: bash\n        if: runner.os == 'Windows'\n        run: tsc --noEmit -p grouped-windows/tsconfig.json\n",
        )],
    };
    let tracked = BTreeSet::from([
        "grouped-linux/tsconfig.json".to_string(),
        "grouped-windows/tsconfig.json".to_string(),
        "known-windows/tsconfig.json".to_string(),
        "linux/tsconfig.json".to_string(),
        "windows/tsconfig.json".to_string(),
        "wrong-linux/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        collect_ci_projects_with_stats(&workflows, &tracked, &project_inputs(&tracked)).0,
        BTreeSet::from([
            "grouped-linux/tsconfig.json".to_string(),
            "grouped-windows/tsconfig.json".to_string(),
            "known-windows/tsconfig.json".to_string(),
            "linux/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn known_runner_os_controls_resolved_environment_urls() {
    let workflows = ParsedWorkflowSet {
        documents: vec![document(
            ".github/workflows/checks.yml",
            "on: push\njobs:\n  invalid-linux:\n    runs-on: ubuntu-latest\n    environment:\n      name: production\n      url: \"${{ case(runner.os == 'Linux', fromJSON('{}'), 'https://ok') }}\"\n    steps:\n      - run: tsc --noEmit -p invalid/tsconfig.json\n  valid-windows:\n    runs-on: windows-latest\n    environment:\n      name: production\n      url: \"${{ case(runner.os == 'Linux', fromJSON('{}'), 'https://ok') }}\"\n    steps:\n      - shell: bash\n        run: tsc --noEmit -p valid/tsconfig.json\n",
        )],
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
