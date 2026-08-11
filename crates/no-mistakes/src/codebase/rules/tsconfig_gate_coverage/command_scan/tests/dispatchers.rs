use super::*;

#[test]
fn shell_scanner_rejects_reachability_control_commands_without_modeling_them() {
    for script in [
        "exit 0; tsc --noEmit --project app/tsconfig.json",
        "false && tsc --noEmit --project app/tsconfig.json",
        "return; tsc --noEmit --project app/tsconfig.json",
        "eval 'set -o pipefail'; tsc --noEmit --project app/tsconfig.json",
        "source shell-options.sh; tsc --noEmit --project app/tsconfig.json",
        ". shell-options.sh; tsc --noEmit --project app/tsconfig.json",
        "command -p -- source shell-options.sh; tsc --noEmit --project app/tsconfig.json",
        "builtin eval 'set -o pipefail'; tsc --noEmit --project app/tsconfig.json",
        "builtin -- . shell-options.sh; tsc --noEmit --project app/tsconfig.json",
        "exec true; tsc --noEmit --project app/tsconfig.json",
        "command -- exec true; tsc --noEmit --project app/tsconfig.json",
        "builtin -s exec true; tsc --noEmit --project app/tsconfig.json",
        "\\eval 'set -o pipefail'; tsc --noEmit --project app/tsconfig.json",
        "command -p \\eval 'set -o pipefail'; tsc --noEmit --project app/tsconfig.json",
        "builtin -- \\source shell-options.sh; tsc --noEmit --project app/tsconfig.json",
        "e\\val 'set -o pipefail'; tsc --noEmit --project app/tsconfig.json",
        "\\command -p \\builtin -- \\source shell-options.sh; tsc --noEmit --project app/tsconfig.json",
        "time external-command; tsc --noEmit --project app/tsconfig.json",
        "if true; then tsc --noEmit --project app/tsconfig.json; fi",
        "for value in one; do tsc --noEmit --project app/tsconfig.json; done",
        "f\"a\"lse; tsc --noEmit --project app/tsconfig.json",
        "s\"et\" -o pipefail; tsc --noEmit --project app/tsconfig.json",
        "$'set' -o pipefail; tsc --noEmit --project app/tsconfig.json",
        "tsc --noEmit --project app/tsconfig.json && exit 0",
        "command X=1 tsc --noEmit --project app/tsconfig.json",
        "command -v tsc --noEmit --project app/tsconfig.json",
        "command -V tsc --noEmit --project app/tsconfig.json",
        "command --bad tsc --noEmit --project app/tsconfig.json",
        "builtin tsc --noEmit --project app/tsconfig.json",
        "command -p tsc --noEmit --project app/tsconfig.json",
        "set -$OPT; tsc --noEmit --project app/tsconfig.json",
        "cd \"$DIR\"; tsc --noEmit --project app/tsconfig.json",
        "X=1 eval \"$CMD\"; tsc --noEmit --project app/tsconfig.json",
        "PATH=./fake tsc --noEmit --project app/tsconfig.json",
        "hash -p ./fake/tsc tsc; tsc --noEmit --project app/tsconfig.json",
        "trap 'exit 0' ERR; false; tsc --noEmit --project app/tsconfig.json",
        "export PATH=./fake; tsc --noEmit --project app/tsconfig.json",
        "set +o pipefail; false | true; tsc --noEmit --project app/tsconfig.json",
        "(false); tsc --noEmit --project app/tsconfig.json",
        "true & false; tsc --noEmit --project app/tsconfig.json",
        "false |& true; tsc --noEmit --project app/tsconfig.json",
        "set -u -e; false; tsc --noEmit --project app/tsconfig.json",
        "set -u +e; false; tsc --noEmit --project app/tsconfig.json",
        "printf -v PATH ./fake; tsc --noEmit --project app/tsconfig.json",
        "mapfile -t PATH; tsc --noEmit --project app/tsconfig.json",
        "true > /definitely-missing/file; tsc --noEmit --project app/tsconfig.json",
        "true 2>> /definitely-missing/file; tsc --noEmit --project app/tsconfig.json",
        "true < /definitely-missing/file; tsc --noEmit --project app/tsconfig.json",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }

    assert_eq!(
        scan_shell_for_typechecked_projects(
            "cd app; tsc --noEmit; cd tools; tsc --noEmit --project tsconfig.tools.json",
            ".",
        ),
        vec!["app/tools/tsconfig.tools.json", "app/tsconfig.json"]
    );
    assert_eq!(
        scan_shell_for_typechecked_projects("bash setup.sh; tsc --noEmit", "."),
        vec!["tsconfig.json"]
    );
    assert_eq!(
        scan_shell_for_typechecked_projects("echo 'still safe'; tsc --noEmit", "."),
        vec!["tsconfig.json"]
    );
    assert_eq!(
        scan_shell_for_typechecked_projects("echo 'literal > text'; tsc --noEmit", "."),
        vec!["tsconfig.json"]
    );
    assert_eq!(
        scan_shell_for_typechecked_projects(
            "command command -- tsc --noEmit --project app/tsconfig.json",
            ".",
        ),
        vec!["app/tsconfig.json"]
    );
}
