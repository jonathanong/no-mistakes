use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf8")
}

#[cfg(unix)]
#[test]
fn lockfile_git_discovery_timeout_does_not_render_partial_output() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/invocation/git-discovery-timeout");
    let fake_bin = fixture.join("bin");
    let root = fixture.join("repo");
    let home = tempfile::tempdir().unwrap();
    let path = std::env::join_paths(std::iter::once(fake_bin).chain(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    )))
    .unwrap();

    let output = Command::new(bin())
        .args([
            "--timeout",
            "1",
            "lockfile",
            "diff",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
        ])
        .env("PATH", path)
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", home.path())
        .output()
        .expect("no-mistakes should run");

    assert_eq!(output.status.code(), Some(124));
    assert!(
        output.stdout.is_empty(),
        "timed-out discovery must not render a partial result: {}",
        stdout(&output)
    );
    assert!(stderr(&output).contains("timed out"));
}

#[test]
fn invocation_help_documents_independent_timeouts_and_lock_failure() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("--timeout <SECONDS>"));
    assert!(help.contains("--lock-timeout <SECONDS>"));
    assert!(help.contains("--fail-on-lock"));
    assert!(help.contains("[default: 30]"));
}

#[test]
fn invocation_options_are_global_at_root_command_and_leaf_boundaries() {
    for args in [
        vec![
            "--timeout",
            "0",
            "--lock-timeout",
            "0",
            "--fail-on-lock",
            "dependencies",
            "--help",
        ],
        vec![
            "dependencies",
            "--timeout",
            "0",
            "--lock-timeout",
            "0",
            "--fail-on-lock",
            "--help",
        ],
        vec![
            "tests",
            "plan",
            "--timeout",
            "0",
            "--lock-timeout",
            "0",
            "--fail-on-lock",
            "--help",
        ],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{} failed: {}",
            args.join(" "),
            stderr(&output)
        );
    }
}

#[test]
fn invocation_timeouts_reject_non_integer_cli_values() {
    for option in ["--timeout", "--lock-timeout"] {
        let output = run(&["dependencies", option, "1.5", "--help"]);

        assert_eq!(output.status.code(), Some(2));
        let error = stderr(&output);
        assert!(error.contains("invalid value '1.5'"), "{error}");
        assert!(error.contains(option), "{error}");
    }
}
