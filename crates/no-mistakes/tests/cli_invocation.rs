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
