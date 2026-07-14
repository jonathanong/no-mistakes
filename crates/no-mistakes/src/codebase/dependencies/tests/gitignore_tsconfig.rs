use assert_cmd::Command;

fn fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
}

fn dependencies(root: &std::path::Path, explicit: bool) -> serde_json::Value {
    let mut command = Command::cargo_bin("no-mistakes").unwrap();
    command
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
