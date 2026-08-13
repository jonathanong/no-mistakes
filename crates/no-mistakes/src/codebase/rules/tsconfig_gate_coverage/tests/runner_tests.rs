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

#[test]
fn ci_scanner_only_credits_implicit_shells_on_known_posix_runners() {
    let workflow: Value = serde_yaml::from_str(
        "on: push\njobs:\n  self-hosted-macos:\n    runs-on: [self-hosted, macOS]\n    steps:\n      - run: tsc --noEmit --project self-hosted-macos/tsconfig.json\n  macos-xlarge:\n    runs-on: xcode-27-xlarge\n    steps:\n      - run: tsc --noEmit --project macos-xlarge/tsconfig.json\n  obsolete-macos-xlarge:\n    runs-on: macos-13-xlarge\n    steps:\n      - run: tsc --noEmit --project obsolete-macos-xlarge/tsconfig.json\n  custom-runner:\n    runs-on: custom-runner\n    steps:\n      - run: tsc --noEmit --project custom-runner/tsconfig.json\n  custom-macos-runner:\n    runs-on: macos-custom\n    steps:\n      - run: tsc --noEmit --project custom-macos-runner/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/runner-posix.yml".into(),
            value: Ok(workflow),
        }],
    };
    let expected = BTreeSet::from([
        "macos-xlarge/tsconfig.json".to_string(),
        "self-hosted-macos/tsconfig.json".to_string(),
    ]);
    let projects = BTreeSet::from([
        "custom-macos-runner/tsconfig.json".to_string(),
        "custom-runner/tsconfig.json".to_string(),
        "macos-xlarge/tsconfig.json".to_string(),
        "obsolete-macos-xlarge/tsconfig.json".to_string(),
        "self-hosted-macos/tsconfig.json".to_string(),
    ]);
    assert_eq!(
        ci_typechecked_projects(&workflows, &projects, &project_inputs(&projects)),
        expected
    );
}

#[test]
fn ci_scanner_distinguishes_truthy_object_matrix_values_from_missing_properties() {
    let workflow: Value = serde_yaml::from_str(
        "on: push\njobs:\n  missing-comparison:\n    runs-on: ubuntu-latest\n    strategy:\n      matrix:\n        cfg: [{package: app}]\n    steps:\n      - if: ${{ matrix.cfg == '' }}\n        run: tsc --noEmit --project object-matrix-missing/tsconfig.json\n  truthy-object:\n    runs-on: ubuntu-latest\n    strategy:\n      matrix:\n        cfg: [{package: app}]\n    steps:\n      - if: ${{ matrix.cfg }}\n        run: tsc --noEmit --project object-matrix-truthy/tsconfig.json\n  negated-object:\n    runs-on: ubuntu-latest\n    strategy:\n      matrix:\n        cfg: [{package: app}]\n    steps:\n      - if: ${{ !matrix.cfg }}\n        run: tsc --noEmit --project object-matrix-negated/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/object-matrix.yml".into(),
            value: Ok(workflow),
        }],
    };
    let projects = BTreeSet::from([
        "object-matrix-missing/tsconfig.json".to_string(),
        "object-matrix-negated/tsconfig.json".to_string(),
        "object-matrix-truthy/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &projects, &project_inputs(&projects)),
        BTreeSet::from(["object-matrix-truthy/tsconfig.json".to_string()])
    );
}

#[test]
fn ci_scanner_resolves_step_timeout_from_step_environment() {
    let workflow: Value = serde_yaml::from_str(
        r#"on: push
env: {TIMEOUT: 3}
jobs:
  typecheck:
    runs-on: ubuntu-latest
    env: {TIMEOUT: 4}
    steps:
      - env: {TIMEOUT: 5}
        timeout-minutes: "${{ fromJSON(env.TIMEOUT) }}"
        run: tsc --noEmit --project step-timeout/tsconfig.json
      - env: {TIMEOUT: 361}
        timeout-minutes: "${{ fromJSON(env.TIMEOUT) }}"
        run: tsc --noEmit --project invalid-step-timeout/tsconfig.json
"#,
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/step-timeout.yml".into(),
            value: Ok(workflow),
        }],
    };
    let projects = BTreeSet::from([
        "invalid-step-timeout/tsconfig.json".to_string(),
        "step-timeout/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &projects, &project_inputs(&projects)),
        BTreeSet::from(["step-timeout/tsconfig.json".to_string()])
    );
}

#[test]
fn ci_scanner_resolves_literal_shell_expressions_without_guessing_dynamic_shells() {
    let workflow: Value = serde_yaml::from_str(
        "on: push\njobs:\n  literal-bash:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: \"${{ 'bash' }}\"\n        run: tsc --noEmit --project literal-bash/tsconfig.json\n  literal-errexit:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: \"${{ 'bash -e {0}' }}\"\n        run: tsc --noEmit --project literal-errexit/tsconfig.json; echo later\n  dynamic-shell:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: '${{ vars.SHELL }}'\n        run: tsc --noEmit --project dynamic-shell/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/shell-expressions.yml".into(),
            value: Ok(workflow),
        }],
    };
    let projects = BTreeSet::from([
        "dynamic-shell/tsconfig.json".to_string(),
        "literal-bash/tsconfig.json".to_string(),
        "literal-errexit/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        ci_typechecked_projects(&workflows, &projects, &project_inputs(&projects)),
        BTreeSet::from([
            "literal-bash/tsconfig.json".to_string(),
            "literal-errexit/tsconfig.json".to_string(),
        ])
    );
}
