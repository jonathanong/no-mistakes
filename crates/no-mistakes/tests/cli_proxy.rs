use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn proxy_fixture(name: &str) -> PathBuf {
    no_mistakes_core::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/no-mistakes-proxy")
            .join(name),
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn proxy_path() -> std::ffi::OsString {
    env::join_paths(
        std::iter::once(proxy_fixture("bin"))
            .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
    )
    .expect("PATH should join")
}

#[test]
fn external_subcommand_proxies_to_no_mistakes_executable_on_path() {
    let output = Command::new(bin())
        .env("PATH", proxy_path())
        .args(["fixture-proxy", "--print", "one"])
        .output()
        .expect("no-mistakes should run");

    assert!(output.status.success());
    assert_eq!(stdout(&output).trim(), "fixture-proxy:--print:one");
}

#[test]
fn external_subcommand_preserves_proxy_exit_status() {
    let output = Command::new(bin())
        .env("PATH", proxy_path())
        .args(["fixture-proxy", "--fail"])
        .output()
        .expect("no-mistakes should run");

    assert_eq!(output.status.code(), Some(7));
    assert!(stdout(&output).contains("proxy failed"));
}

#[test]
fn external_subcommand_reports_missing_executable() {
    let output = Command::new(bin())
        .env("PATH", "")
        .arg("missing-proxy")
        .output()
        .expect("no-mistakes should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no-mistakes-missing-proxy"));
}
