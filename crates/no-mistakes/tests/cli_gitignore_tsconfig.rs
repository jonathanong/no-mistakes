use assert_cmd::Command;

fn fixture() -> tempfile::TempDir {
    let fixture = crate::gitignore_fixture::materialize("prepared-tsconfig");
    git(fixture.path(), &["init", "-q", "--initial-branch=main"]);
    git(fixture.path(), &["add", "."]);
    fixture
}

fn dependencies(root: &std::path::Path, explicit: bool) -> serde_json::Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .env(
            "CARGO_BIN_EXE_no-mistakes",
            env!("CARGO_BIN_EXE_no-mistakes"),
        )
        .arg("dependencies")
        .arg("entry.ts")
        .arg("--root")
        .arg(root)
        .arg("--relationship")
        .arg("import")
        .arg("--format")
        .arg("json");
    if explicit {
        command.arg("--tsconfig").arg("tsconfig.json");
    }
    let output = command.assert().success().get_output().stdout.clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn cli_ignores_automatic_tsconfig_but_honors_explicit_ignored_config() {
    let fixture = fixture();
    let automatic = dependencies(fixture.path(), false);
    let automatic = automatic["files"].as_array().unwrap();
    assert!(automatic
        .iter()
        .any(|row| row["module"] == "@lib/forbidden"));
    assert!(!automatic
        .iter()
        .any(|row| row["path"] == "src/forbidden.ts"));

    let explicit = dependencies(fixture.path(), true);
    let explicit = explicit["files"].as_array().unwrap();
    assert!(explicit.iter().any(|row| row["path"] == "src/forbidden.ts"));
    assert!(!explicit.iter().any(|row| row["module"] == "@lib/forbidden"));
}

fn git(root: &std::path::Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
