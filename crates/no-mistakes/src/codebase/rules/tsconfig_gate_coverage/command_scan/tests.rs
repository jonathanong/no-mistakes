use super::*;

#[test]
fn shell_scanner_tracks_cd_and_pnpm_dir() {
    assert_eq!(
        scan_shell_for_typechecked_projects(
            "cd app && pnpm exec tsc --noEmit; pnpm --dir tools exec tsc --noEmit --project tsconfig.tools.json",
            ".",
        ),
        vec!["app/tools/tsconfig.tools.json", "app/tsconfig.json"]
    );
}

#[test]
fn dynamic_and_indirect_commands_do_not_count() {
    for script in [
        "\"$ROOT_BIN/tsc\" --noEmit --project app/tsconfig.json",
        "runner tsc --noEmit --project app/tsconfig.json",
        "tsc --noEmit | tee result",
        "tsc --noEmit || exit 1",
        "'tsc'--noEmit",
        "'unterminated",
        "''",
        "cd app ignored && tsc --noEmit",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }
    assert!(scan_shell_for_typechecked_projects("tsc --noEmit", "../outside").is_empty());
    assert!(scan_shell_for_typechecked_projects("cd ../outside && tsc --noEmit", ".").is_empty());
    assert!(scan_shell_for_typechecked_projects("tsc --project app/tsconfig.json", ".").is_empty());
}

#[test]
fn argv_scanner_handles_static_forms_and_rejects_ambiguous_projects() {
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "pnpm".into(),
                "--dir=app".into(),
                "exec".into(),
                "./node_modules/.bin/tsc".into(),
                "--noEmit".into(),
            ],
            ".",
        ),
        vec!["app/tsconfig.json"]
    );
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "--project=app/tsconfig.json".into()
            ],
            ".",
        ),
        vec!["app/tsconfig.json"]
    );
    for argv in [
        vec!["tsc".into(), "--noEmit".into(), "--project".into()],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "--project".into(),
            "a/tsconfig.json".into(),
            "--project".into(),
            "b/tsconfig.json".into(),
        ],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "-p".into(),
            "app/tsconfig.json".into(),
        ],
    ] {
        assert!(scan_argv_for_typechecked_projects(&argv, ".").is_empty());
    }
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

#[test]
fn argv_scanner_supports_shell_and_rejects_invalid_wrappers() {
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "sh".into(),
                "-c".into(),
                "tsc --noEmit --project app/tsconfig.json".into()
            ],
            ".",
        ),
        vec!["app/tsconfig.json"]
    );
    for (argv, cwd) in [
        (
            vec![
                "bash".into(),
                "-c".into(),
                "tsc --noEmit".into(),
                "extra".into(),
            ],
            ".",
        ),
        (
            vec![
                "pnpm".into(),
                "exec".into(),
                "tsc".into(),
                "--noEmit".into(),
            ],
            "../outside",
        ),
        (
            vec![
                "pnpm".into(),
                "--dir".into(),
                "../outside".into(),
                "exec".into(),
                "tsc".into(),
                "--noEmit".into(),
            ],
            ".",
        ),
    ] {
        assert!(scan_argv_for_typechecked_projects(&argv, cwd).is_empty());
    }
}

#[test]
fn token_scanner_covers_pnpm_and_project_argument_edge_cases() {
    let tsc = vec!["tsc".into(), "--noEmit".into()];
    assert_eq!(scan_tokens(&tsc, "tools"), vec!["tools/tsconfig.json"]);
    for tokens in [
        vec!["pnpm".into()],
        vec!["pnpm".into(), "--dir".into()],
        vec!["pnpm".into(), "run".into(), "tsc".into(), "--noEmit".into()],
    ] {
        assert!(scan_tokens(&tokens, ".").is_empty());
    }
    assert_eq!(
        project_argument(&["tsc".into(), "--project=app/tsconfig.json".into()]),
        Some("app/tsconfig.json".into())
    );
    assert_eq!(project_argument(&tsc), Some("tsconfig.json".into()));
    assert_eq!(
        join_relative(".", "./app/tsconfig.json"),
        Some("app/tsconfig.json".into())
    );
    assert_eq!(join_relative("app", "../tsconfig.json"), None);
}
