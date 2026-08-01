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
fn shell_scanner_rejects_separators_inside_quoted_literals() {
    for script in [
        "echo '; tsc --noEmit --project app/tsconfig.json; :'",
        "echo \"&& tsc --noEmit --project app/tsconfig.json\"",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }
}

#[test]
fn shell_scanner_rejects_unmodeled_function_and_group_bodies() {
    for script in [
        "typecheck() {\n  tsc --noEmit --project app/tsconfig.json\n}",
        "function typecheck { tsc --noEmit --project app/tsconfig.json; }",
        "{ tsc --noEmit --project app/tsconfig.json; }",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
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

#[test]
fn informational_or_init_tsc_modes_do_not_count_as_typechecks() {
    for mode in [
        "--showConfig",
        "--help",
        "-h",
        "--version",
        "-v",
        "--init",
        "--noCheck",
        "--noCheck=true",
        "--noCheck=maybe",
        "--listFilesOnly",
        "--listFilesOnly=true",
        "--ignoreConfig",
        "--ignoreConfig=true",
    ] {
        assert!(scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                mode.into(),
                "--project".into(),
                "app/tsconfig.json".into(),
            ],
            ".",
        )
        .is_empty());
    }

    for no_check in [
        vec!["--noCheck".into(), "false".into()],
        vec!["--noCheck=false".into()],
    ] {
        let argv = [
            vec!["tsc".into(), "--noEmit".into()],
            no_check,
            vec!["--project".into(), "app/tsconfig.json".into()],
        ]
        .concat();
        assert_eq!(
            scan_argv_for_typechecked_projects(&argv, "."),
            vec!["app/tsconfig.json"]
        );
    }
    assert!(scan_argv_for_typechecked_projects(
        &[
            "tsc".into(),
            "--noEmit".into(),
            "--noCheck".into(),
            "true".into(),
            "--project".into(),
            "app/tsconfig.json".into(),
        ],
        ".",
    )
    .is_empty());
}

#[test]
fn normalizer_accepts_only_static_relative_paths() {
    assert_eq!(
        normalize_repo_relative("./app//tsconfig.json"),
        Some("app/tsconfig.json".into())
    );
    for invalid in [
        "",
        "/tmp/tsconfig.json",
        "~/tsconfig.json",
        "../tsconfig.json",
        "$ROOT/tsconfig.json",
        "app\\tsconfig.json",
        "$(pwd)/tsconfig.json",
    ] {
        assert_eq!(normalize_repo_relative(invalid), None, "{invalid}");
    }
    assert_eq!(normalize_repo_relative("./"), Some(".".into()));
}

#[test]
fn tokenizer_leaves_whitespace_only_input_without_a_command() {
    assert_eq!(static_tokens(" \t"), Some(Vec::new()));
}
