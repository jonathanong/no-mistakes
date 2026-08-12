use super::*;

#[test]
fn matrix_object_identity_keeps_self_comparisons_true_and_distinct_values_false() {
    let workflow = document(
        ".github/workflows/matrix-identity.yml",
        "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix:\n        include:\n          - cfg: {package: app, enabled: false}\n            other: {package: app}\n    runs-on: ubuntu-latest\n    steps:\n      - if: matrix.cfg == matrix.cfg\n        run: tsc --noEmit -p self/tsconfig.json\n      - if: matrix.cfg == matrix.other\n        run: tsc --noEmit -p distinct/tsconfig.json\n      - if: matrix.cfg.enabled == false\n        run: tsc --noEmit -p nested/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["self", "distinct", "nested"]),
        BTreeSet::from([
            "self/tsconfig.json".to_string(),
            "nested/tsconfig.json".to_string()
        ])
    );
}
