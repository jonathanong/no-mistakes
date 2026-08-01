use super::super::*;

#[test]
fn shell_scanner_respects_static_comment_boundaries() {
    for script in [
        "# disabled; tsc --noEmit",
        "# disabled && tsc --noEmit",
        "tsc --noEmit # project gate",
        "echo foo#bar; tsc --noEmit",
        "# <<EOF; tsc --noEmit",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }
    for script in ["echo '# literal'; tsc --noEmit", "echo \\#; tsc --noEmit"] {
        assert_eq!(
            scan_shell_for_typechecked_projects(script, "."),
            vec!["tsconfig.json"],
            "{script}"
        );
    }
}

#[test]
fn local_shell_scanner_rejects_non_executing_modes() {
    for argv in [
        vec![
            "bash".into(),
            "-n".into(),
            "-c".into(),
            "tsc --noEmit".into(),
        ],
        vec!["sh".into(), "-nc".into(), "tsc --noEmit".into()],
        vec!["bash".into(), "-c".into(), "set -n; tsc --noEmit".into()],
        vec![
            "bash".into(),
            "-c".into(),
            "set -o noexec; tsc --noEmit".into(),
        ],
    ] {
        assert!(
            scan_argv_for_typechecked_projects(&argv, ".").is_empty(),
            "{argv:?}"
        );
    }
}
