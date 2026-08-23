use super::*;

#[test]
fn fail_fast_matrix_failures_do_not_credit_cancellable_instances() {
    let workflow = document(
        ".github/workflows/matrix.yml",
        "on: push\njobs:\n  default-fail-fast:\n    strategy:\n      matrix: {project: [first, second]}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p '${{ matrix.project }}/tsconfig.json'\n      - if: matrix.project == 'first'\n        run: exit 1\n  disabled-fail-fast:\n    strategy:\n      fail-fast: false\n      matrix: {project: [third, fourth]}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p '${{ matrix.project }}/tsconfig.json'\n      - if: matrix.project == 'third'\n        run: exit 1\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["first", "second", "third", "fourth"]),
        BTreeSet::from([
            "first/tsconfig.json".to_string(),
            "fourth/tsconfig.json".to_string(),
            "third/tsconfig.json".to_string(),
        ])
    );
}
