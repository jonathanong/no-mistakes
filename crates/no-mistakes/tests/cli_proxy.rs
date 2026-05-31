use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf8")
}

#[test]
fn unknown_commands_are_rejected_without_external_proxy() {
    let output = Command::new(bin())
        .arg("fixture-proxy")
        .output()
        .expect("no-mistakes should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("unrecognized subcommand 'fixture-proxy'"));
}

#[test]
fn old_standalone_command_names_are_not_accepted_as_subcommands() {
    for command in ["queue-ast-hop", "react-traits", "server-ast-routes"] {
        let output = Command::new(bin())
            .arg(command)
            .output()
            .expect("no-mistakes should run");

        assert_eq!(output.status.code(), Some(2), "{command}");
        assert!(
            stderr(&output).contains("unrecognized subcommand"),
            "{command}"
        );
    }
}
