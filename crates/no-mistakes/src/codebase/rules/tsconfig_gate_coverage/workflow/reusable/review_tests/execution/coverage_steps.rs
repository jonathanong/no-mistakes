use super::*;

#[test]
fn malformed_and_unresolved_steps_stop_coverage_at_the_runtime_boundary() {
    let cases = [
        (
            "invalid-condition",
            "      - if: ${{ fromJSON('not-json') }}\n        run: tsc --noEmit -p invalid-condition/tsconfig.json\n",
        ),
        (
            "invalid-environment",
            "      - env: {VALUE: \"${{ fromJSON('[]') }}\"}\n        run: tsc --noEmit -p invalid-environment/tsconfig.json\n",
        ),
        (
            "invalid-action-input",
            "      - uses: actions/checkout@v4\n        with: {ref: \"${{ fromJSON('{}') }}\"}\n      - run: tsc --noEmit -p invalid-action-input/tsconfig.json\n",
        ),
        (
            "unavailable-local-action",
            "      - uses: ./local-action\n      - run: tsc --noEmit -p unavailable-local-action/tsconfig.json\n",
        ),
        (
            "tolerated-missing-directory",
            "      - continue-on-error: true\n        working-directory: missing\n        run: echo absent\n      - run: tsc --noEmit -p tolerated/tsconfig.json\n",
        ),
        (
            "unresolved-run",
            "      - run: \"${{ github.event.unknown }}\"\n      - run: tsc --noEmit -p unresolved-run/tsconfig.json\n",
        ),
        (
            "tolerated-unresolved-run",
            "      - continue-on-error: true\n        run: \"${{ github.event.unknown }}\"\n      - run: tsc --noEmit -p tolerated-run/tsconfig.json\n",
        ),
        (
            "unresolved-shell",
            "      - shell: \"${{ github.event.unknown }}\"\n        run: echo unresolved\n      - run: tsc --noEmit -p unresolved-shell/tsconfig.json\n",
        ),
        (
            "tolerated-unresolved-shell",
            "      - continue-on-error: true\n        shell: \"${{ github.event.unknown }}\"\n        run: echo unresolved\n      - run: tsc --noEmit -p tolerated-shell/tsconfig.json\n",
        ),
    ];
    let workflows = cases
        .into_iter()
        .map(|(name, steps)| {
            document(
                &format!(".github/workflows/{name}.yml"),
                &format!(
                    "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n{steps}"
                ),
            )
        })
        .collect();

    assert_eq!(
        scanned_projects(
            workflows,
            &[
                "invalid-condition",
                "invalid-environment",
                "invalid-action-input",
                "unavailable-local-action",
                "tolerated",
                "unresolved-run",
                "tolerated-run",
                "unresolved-shell",
                "tolerated-shell",
            ],
        ),
        BTreeSet::from([
            "invalid-condition/tsconfig.json".to_string(),
            "tolerated/tsconfig.json".to_string(),
            "tolerated-run/tsconfig.json".to_string(),
            "tolerated-shell/tsconfig.json".to_string(),
        ])
    );
}
