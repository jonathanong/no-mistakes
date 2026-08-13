use super::*;

#[test]
fn shell_dispatchers_only_credit_commands_that_execute_the_selected_target() {
    let workflow = document(
        ".github/workflows/dispatchers.yml",
        "on: push\njobs:\n  assignment-target:\n    runs-on: ubuntu-latest\n    steps:\n      - run: command X=1 tsc --noEmit -p assignment-target/tsconfig.json\n  query:\n    runs-on: ubuntu-latest\n    steps:\n      - run: command -v tsc --noEmit -p query/tsconfig.json\n  builtin:\n    runs-on: ubuntu-latest\n    steps:\n      - run: builtin tsc --noEmit -p builtin/tsconfig.json\n  path-control:\n    runs-on: ubuntu-latest\n    steps:\n      - run: command -p tsc --noEmit -p path-control/tsconfig.json\n  assigned-path:\n    runs-on: ubuntu-latest\n    steps:\n      - run: PATH=./fake tsc --noEmit -p assigned-path/tsconfig.json\n  hashed-path:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'hash -p ./fake/tsc tsc; tsc --noEmit -p hashed-path/tsconfig.json'\n  recursive:\n    runs-on: ubuntu-latest\n    steps:\n      - run: command command -- tsc --noEmit -p recursive/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "assignment-target",
                "query",
                "builtin",
                "path-control",
                "assigned-path",
                "hashed-path",
                "recursive"
            ],
        ),
        BTreeSet::from(["recursive/tsconfig.json".to_string()])
    );
}

#[test]
fn indeterminate_commands_do_not_manufacture_failure_or_later_success() {
    let workflow = document(
        ".github/workflows/indeterminate.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: command -v true\n  ordinary:\n    needs: setup\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p ordinary/tsconfig.json\n  skipped-handler:\n    needs: ordinary\n    if: always() && needs.ordinary.result == 'skipped'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p skipped-handler/tsconfig.json\n  failure:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure/tsconfig.json\n  always:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always/tsconfig.json\n  or-list:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'false || false; tsc --noEmit -p or-list/tsconfig.json'\n  or-pipe:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'false | true || false; tsc --noEmit -p or-pipe/tsconfig.json'\n  dynamic-set:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'set -$OPT; tsc --noEmit -p dynamic-set/tsconfig.json'\n  dynamic-cd:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'cd \"$DIR\"; tsc --noEmit -p dynamic-cd/tsconfig.json'\n  assigned-eval:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 'X=1 eval \"$CMD\"; tsc --noEmit -p assigned-eval/tsconfig.json'\n",
    );

    assert_eq!(
        scanned_projects(
            vec![workflow],
            &[
                "ordinary",
                "skipped-handler",
                "failure",
                "always",
                "or-list",
                "or-pipe",
                "dynamic-set",
                "dynamic-cd",
                "assigned-eval",
            ]
        ),
        BTreeSet::from(["always/tsconfig.json".to_string()])
    );
}

#[test]
fn indeterminate_local_reusable_results_only_allow_guaranteed_continuations() {
    let caller = document(
        ".github/workflows/caller.yml",
        "on: push\njobs:\n  call:\n    uses: ./.github/workflows/callee.yml\n  ordinary:\n    needs: call\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p ordinary/tsconfig.json\n  failure:\n    needs: call\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure/tsconfig.json\n  always:\n    needs: call\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always/tsconfig.json\n",
    );
    let callee = document(
        ".github/workflows/callee.yml",
        "on: workflow_call\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - run: command -v true\n",
    );

    assert_eq!(
        scanned_projects(vec![caller, callee], &["ordinary", "failure", "always"],),
        BTreeSet::from(["always/tsconfig.json".to_string()])
    );
}

#[test]
fn runtime_shell_mutations_do_not_manufacture_failure_results() {
    let workflow = document(
        ".github/workflows/runtime-mutation.yml",
        "on: push\njobs:\n  setup:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: 'set +o pipefail; false | true'\n  failure:\n    needs: setup\n    if: failure()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p failure/tsconfig.json\n  always:\n    needs: setup\n    if: always()\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit -p always/tsconfig.json\n",
    );

    assert_eq!(
        scanned_projects(vec![workflow], &["failure", "always"]),
        BTreeSet::from(["always/tsconfig.json".to_string()])
    );
}
