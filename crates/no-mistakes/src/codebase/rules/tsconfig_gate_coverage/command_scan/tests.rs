use super::*;

mod review;

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
fn shell_scanner_skips_whitespace_only_segments() {
    assert_eq!(
        scan_shell_for_typechecked_projects("  \n tsc --noEmit", "."),
        vec!["tsconfig.json"]
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
fn shell_scanner_rejects_reachability_control_commands_without_modeling_them() {
    for script in [
        "exit 0; tsc --noEmit --project app/tsconfig.json",
        "false && tsc --noEmit --project app/tsconfig.json",
        "return; tsc --noEmit --project app/tsconfig.json",
        "tsc --noEmit --project app/tsconfig.json && exit 0",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }

    assert_eq!(
        scan_shell_for_typechecked_projects(
            "cd app && tsc --noEmit; cd tools && tsc --noEmit --project tsconfig.tools.json",
            ".",
        ),
        vec!["app/tools/tsconfig.tools.json", "app/tsconfig.json"]
    );
}

#[test]
fn shell_scanner_rejects_failure_enforcement_mutations() {
    for script in [
        "set +e; tsc --noEmit --project app/tsconfig.json",
        "set +eu; tsc --noEmit --project app/tsconfig.json",
        "set +o errexit; tsc --noEmit --project app/tsconfig.json",
        "tsc --noEmit --project app/tsconfig.json; set +e",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }
    assert_eq!(
        scan_shell_for_typechecked_projects("set -e; tsc --noEmit", "."),
        vec!["tsconfig.json"]
    );
    assert_eq!(
        scan_shell_for_typechecked_projects("set -o errexit; tsc --noEmit", "."),
        vec!["tsconfig.json"]
    );
    assert_eq!(
        scan_shell_for_typechecked_projects("set -u; tsc --noEmit", "."),
        vec!["tsconfig.json"]
    );
}

#[test]
fn shell_scanner_rejects_unsupported_working_directory_commands() {
    for script in [
        "pushd app; tsc --noEmit",
        "tsc --noEmit; popd",
        "dirs; tsc --noEmit",
        "cd app ignored; tsc --noEmit",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }
}

#[test]
fn local_shell_gates_require_failure_propagation_or_a_final_typecheck() {
    for argv in [
        vec![
            "bash".into(),
            "-c".into(),
            "tsc --noEmit; echo ignored failure".into(),
        ],
        vec![
            "sh".into(),
            "-c".into(),
            "tsc --noEmit && echo success".into(),
        ],
    ] {
        assert!(
            scan_argv_for_typechecked_projects(&argv, ".").is_empty(),
            "{argv:?}"
        );
    }

    for argv in [
        vec!["bash".into(), "-c".into(), "cd app; tsc --noEmit".into()],
        vec![
            "bash".into(),
            "-ec".into(),
            "cd app; tsc --noEmit; echo reached only after a passing typecheck".into(),
        ],
        vec![
            "sh".into(),
            "-o".into(),
            "errexit".into(),
            "-c".into(),
            "cd app; tsc --noEmit; echo reached only after a passing typecheck".into(),
        ],
        vec![
            "sh".into(),
            "-c".into(),
            "set -e; cd app; tsc --noEmit; echo reached only after a passing typecheck".into(),
        ],
    ] {
        assert_eq!(
            scan_argv_for_typechecked_projects(&argv, "."),
            vec!["app/tsconfig.json"],
            "{argv:?}"
        );
    }
}

#[test]
fn local_shell_parser_rejects_ambiguous_options_and_tracks_errexit() {
    let to_argv = |values: &[&str]| -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    };

    assert_eq!(
        local_shell_command(&to_argv(&["sh", "-o", "errexit", "-c", "script"])),
        Some(("script", true))
    );
    assert_eq!(
        local_shell_command(&to_argv(&["bash", "+e", "-c", "script"])),
        Some(("script", false))
    );
    assert_eq!(
        local_shell_command(&to_argv(&["bash", "+o", "errexit", "-c", "script"])),
        Some(("script", false))
    );
    assert_eq!(
        local_shell_command(&to_argv(&["bash", "-ec", "script"])),
        Some(("script", true))
    );
    assert_eq!(
        local_shell_command(&to_argv(&["bash", "-u", "-c", "script"])),
        Some(("script", false))
    );
    for values in [
        ["node", "-c", "script"].as_slice(),
        ["bash", "-c", "script", "extra"].as_slice(),
        ["bash", "-o"].as_slice(),
        ["bash", "+o", "nounset", "-c", "script"].as_slice(),
        ["bash", "command"].as_slice(),
        ["bash", "-"].as_slice(),
        ["bash", "-1"].as_slice(),
        ["bash", "+c", "script"].as_slice(),
        ["bash", "-e"].as_slice(),
    ] {
        assert_eq!(local_shell_command(&to_argv(values)), None, "{values:?}");
    }
}

#[test]
fn shell_scanner_rejects_heredocs_and_multiline_quoted_bodies() {
    for script in [
        "cat <<'SCRIPT'\ntsc --noEmit\nSCRIPT",
        "tsc --noEmit <<EOF\ninput\nEOF",
        "echo 'data line\ntsc --noEmit\n'",
        "echo \"data line\ntsc --noEmit\n\"",
    ] {
        assert!(
            scan_shell_for_typechecked_projects(script, ".").is_empty(),
            "{script}"
        );
    }
}

#[test]
fn argv_scanner_handles_static_forms_and_rejects_ambiguous_projects() {
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "--module=NodeNext".into(),
                "--pretty=true".into(),
                "--incremental".into(),
                "--composite=false".into(),
                "--skipLibCheck".into(),
            ],
            ".",
        ),
        vec!["tsconfig.json"]
    );
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "-p".into(),
                "app/tsconfig.json".into(),
            ],
            ".",
        ),
        vec!["app/tsconfig.json"]
    );
    for argv in [
        vec!["tsc".into(), "--noEmit".into(), "--project=".into()],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "--module".into(),
            "--skipLibCheck".into(),
        ],
        vec!["tsc".into(), "--noEmit".into(), "--pretty=".into()],
        vec!["tsc".into(), "--noEmit".into(), "--pretty=maybe".into()],
        vec!["tsc".into(), "--noEmit".into(), "-p".into()],
        vec!["tsc".into(), "--noEmit".into(), "-p=".into()],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "--project=app/tsconfig.json".into(),
        ],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "-p=app/tsconfig.json".into(),
        ],
    ] {
        assert!(
            scan_argv_for_typechecked_projects(&argv, ".").is_empty(),
            "{argv:?}"
        );
    }
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
    for command in ["./scripts/tsc", "tools/tsc", "vendor/bin/tsc"] {
        assert!(
            scan_argv_for_typechecked_projects(
                &[
                    "pnpm".into(),
                    "exec".into(),
                    command.into(),
                    "--noEmit".into(),
                ],
                ".",
            )
            .is_empty(),
            "{command}"
        );
    }
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "--project".into(),
                "app".into(),
            ],
            ".",
        ),
        vec!["app"]
    );
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "pnpm".into(),
                "--dir=app".into(),
                "exec".into(),
                "tsc".into(),
                "--noEmit".into(),
                "--project".into(),
                ".".into(),
            ],
            ".",
        ),
        vec!["app"]
    );
    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "--project".into(),
                "app/tsconfig.json".into()
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
    ] {
        assert!(scan_argv_for_typechecked_projects(&argv, ".").is_empty());
    }
}

#[test]
fn default_project_requires_project_mode_without_source_inputs() {
    for argv in [
        vec!["tsc".into(), "--noEmit".into(), "src/main.ts".into()],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "--project".into(),
            "app/tsconfig.json".into(),
            "src/main.ts".into(),
        ],
        vec![
            "tsc".into(),
            "--noEmit".into(),
            "--mystery".into(),
            "value".into(),
        ],
    ] {
        assert!(
            scan_argv_for_typechecked_projects(&argv, ".").is_empty(),
            "{argv:?}"
        );
    }

    assert_eq!(
        scan_argv_for_typechecked_projects(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "--pretty".into(),
                "false".into(),
                "--module".into(),
                "NodeNext".into(),
                "--skipLibCheck".into(),
            ],
            ".",
        ),
        vec!["tsconfig.json"]
    );
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
        project_argument(&[
            "--noEmit".into(),
            "--project".into(),
            "app/tsconfig.json".into(),
        ]),
        Some("app/tsconfig.json".into())
    );
    assert_eq!(
        project_argument(&["--noEmit".into()]),
        Some("tsconfig.json".into())
    );
    assert_eq!(
        scan_tokens(
            &[
                "tsc".into(),
                "--noEmit".into(),
                "--project".into(),
                ".".into()
            ],
            ".",
        ),
        vec!["."]
    );
    assert_eq!(
        join_relative(".", "./app/tsconfig.json"),
        Some("app/tsconfig.json".into())
    );
    assert_eq!(join_relative("app", "../tsconfig.json"), None);
}
