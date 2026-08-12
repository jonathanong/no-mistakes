use super::*;

#[test]
fn unfiltered_and_wildcard_push_refs_are_known_branches_through_reusable_calls() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  direct-tag:\n    if: github.ref == 'refs/tags/v1'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p direct-tag/tsconfig.json\n  direct-branch:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p direct-branch/tsconfig.json\n  call:\n    uses: ./.github/workflows/callee.yml\n",
    );
    let wildcard = document(
        ".github/workflows/wildcard.yml",
        "on:\n  push:\n    branches: ['**']\njobs:\n  tag:\n    if: github.ref == 'refs/tags/v1'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p wildcard-tag/tsconfig.json\n  branch:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p wildcard-branch/tsconfig.json\n",
    );
    let callee = document(
        ".github/workflows/callee.yml",
        "on: workflow_call\njobs:\n  tag:\n    if: github.ref == 'refs/tags/v1'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p callee-tag/tsconfig.json\n  branch:\n    if: github.ref == 'refs/heads/main'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p callee-branch/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![caller, wildcard, callee],
            &[
                "direct-tag",
                "direct-branch",
                "wildcard-tag",
                "wildcard-branch",
                "callee-tag",
                "callee-branch",
            ],
        ),
        BTreeSet::from([
            "callee-branch/tsconfig.json".to_string(),
            "direct-branch/tsconfig.json".to_string(),
            "wildcard-branch/tsconfig.json".to_string(),
        ])
    );
}
