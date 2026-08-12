use super::*;

#[test]
fn tolerated_missing_working_directory_records_failure_outcome_and_success_conclusion() {
    let workflow = document(
        ".github/workflows/tolerated-missing-directory.yml",
        "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - id: missing\n        continue-on-error: true\n        working-directory: missing\n        run: echo setup\n      - if: steps.missing.outcome == 'failure' && steps.missing.conclusion == 'success'\n        run: tsc --noEmit -p recovered/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["recovered"]),
        BTreeSet::from(["recovered/tsconfig.json".to_string()])
    );
}

#[test]
fn unresolved_enforcing_run_interpolations_block_later_and_dependent_typechecks() {
    let workflow = document(
        ".github/workflows/unresolved-run.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ${{ github.event.inputs.unresolved }}\n      - run: tsc --noEmit -p later/tsconfig.json\n  dependent:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n",
    );

    assert!(scanned_projects(vec![workflow], &["later", "dependent"]).is_empty());
}
