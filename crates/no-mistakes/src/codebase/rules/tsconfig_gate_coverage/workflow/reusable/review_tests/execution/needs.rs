use super::*;

#[test]
fn dynamically_conditioned_prerequisites_do_not_unlock_dependents() {
    let workflow = document(
        ".github/workflows/dynamic-prerequisite.yml",
        "on: push\njobs:\n  prerequisite:\n    if: vars.ENABLED\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo setup\n  dependent:\n    needs: prerequisite\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p dependent/tsconfig.json\n  explicit-continuation:\n    needs: prerequisite\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p explicit/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["dependent", "explicit"]),
        BTreeSet::from(["explicit/tsconfig.json".to_string()])
    );
}
