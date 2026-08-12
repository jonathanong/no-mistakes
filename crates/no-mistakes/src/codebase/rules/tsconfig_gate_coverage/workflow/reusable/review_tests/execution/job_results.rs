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
