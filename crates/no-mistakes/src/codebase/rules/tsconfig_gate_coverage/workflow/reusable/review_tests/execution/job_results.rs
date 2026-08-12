use super::*;

#[test]
fn job_level_continue_on_error_publishes_success_to_needs() {
    let workflow = document(
        ".github/workflows/tolerated-job.yml",
        "on: push\njobs:\n  tolerated:\n    continue-on-error: true\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n  after-tolerated:\n    needs: tolerated\n    if: needs.tolerated.result == 'success'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p after-tolerated/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["after-tolerated"]),
        BTreeSet::from(["after-tolerated/tsconfig.json".to_string()])
    );
}

#[test]
fn dynamic_prerequisites_and_tolerated_step_conclusions_gate_dependents() {
    let workflow = document(
        ".github/workflows/results.yml",
        "on: push\njobs:\n  conditional:\n    if: vars.RUN_SETUP\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  after-conditional:\n    needs: conditional\n    runs-on: ubuntu-latest\n    steps:\n      # The prerequisite may be skipped at runtime.\n      - run: tsc --noEmit -p after-conditional/tsconfig.json\n  tolerated:\n    runs-on: ubuntu-latest\n    steps:\n      - id: setup\n        continue-on-error: true\n        run: exit 1\n      - if: steps.setup.outcome == 'failure'\n        run: tsc --noEmit -p outcome/tsconfig.json\n      # Tolerance changes conclusion, but not outcome.\n      - if: steps.setup.conclusion == 'failure'\n        run: tsc --noEmit -p conclusion/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &["after-conditional", "outcome", "conclusion"]
        ),
        BTreeSet::from(["outcome/tsconfig.json".to_string()])
    );
}
