use super::*;

#[test]
fn ci_scanner_requires_static_runners_and_shell_failure_propagation() {
    let workflow: Value = serde_yaml::from_str(
        "on: push\njobs:\n  implicit-shell:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project implicit-shell/tsconfig.json; echo later\n  builtin-bash:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: tsc --noEmit --project builtin-bash/tsconfig.json; echo later\n  custom-final-typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: echo first; tsc --noEmit --project custom-final-typecheck/tsconfig.json\n  custom-masked-typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: tsc --noEmit --project custom-masked-typecheck/tsconfig.json; echo later\n  custom-errexit:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -e {0}'\n        run: tsc --noEmit --project custom-errexit/tsconfig.json; echo later\n  custom-errexit-option:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'sh -o errexit {0}'\n        run: tsc --noEmit --project custom-errexit-option/tsconfig.json; echo later\n  dynamic-runner:\n    runs-on: ${{ matrix.os }}\n    steps:\n      - run: tsc --noEmit --project dynamic-runner/tsconfig.json\n  bare-self-hosted:\n    runs-on: self-hosted\n    steps:\n      - run: tsc --noEmit --project bare-self-hosted/tsconfig.json\n  label-array-runner:\n    runs-on: [self-hosted, linux]\n    steps:\n      - run: tsc --noEmit --project label-array-runner/tsconfig.json\n  dynamic-label-array-runner:\n    runs-on: [self-hosted, '${{ matrix.os }}']\n    steps:\n      - run: tsc --noEmit --project dynamic-label-array-runner/tsconfig.json\n  implicit-windows:\n    runs-on: Windows-2025\n    steps:\n      - run: tsc --noEmit --project implicit-windows/tsconfig.json\n  implicit-self-hosted-windows:\n    runs-on: [self-hosted, windows]\n    steps:\n      - run: tsc --noEmit --project implicit-self-hosted-windows/tsconfig.json\n  explicit-bash-windows:\n    runs-on: windows-latest\n    steps:\n      - shell: bash\n        run: tsc --noEmit --project explicit-bash-windows/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/runners-and-shells.yml".into(),
            value: Ok(workflow),
        }],
    };
    let expected = BTreeSet::from([
        "builtin-bash/tsconfig.json".to_string(),
        "custom-errexit-option/tsconfig.json".to_string(),
        "custom-errexit/tsconfig.json".to_string(),
        "custom-final-typecheck/tsconfig.json".to_string(),
        "explicit-bash-windows/tsconfig.json".to_string(),
        "implicit-shell/tsconfig.json".to_string(),
        "label-array-runner/tsconfig.json".to_string(),
    ]);
    assert_eq!(
        ci_typechecked_projects(&workflows, &expected, &project_inputs(&expected)),
        expected
    );
}
