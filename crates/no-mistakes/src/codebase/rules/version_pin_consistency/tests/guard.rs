use super::*;

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn assert_no_leak(findings: &[RuleFinding]) {
    let blob = format!("{findings:?}");
    assert!(
        !blob.contains("leaked-secret-pin"),
        "external pin leaked: {blob}"
    );
}

#[test]
fn absolute_source_file_does_not_leak_external_contents() {
    let root = fixture("external-pin/repo");
    let outside = fixture("external-pin").join("outside.toml");
    let yaml = OPTIONS.replace(
        "sourceFile: .mise.toml",
        &format!("sourceFile: {}", outside.display()),
    );
    assert_no_leak(&run(&root, &yaml, &PAIR));
}

#[cfg(unix)]
#[test]
fn symlink_source_outside_repo_is_not_read() {
    let root = fixture("symlink-pin/repo");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: pin.toml");
    assert_no_leak(&run(&root, &yaml, &["pin.toml", PAIR[1]]));
}

#[cfg(unix)]
#[test]
fn directory_symlink_escape_is_not_read() {
    let root = fixture("dir-symlink/repo");
    let yaml = OPTIONS.replace("sourceFile: .mise.toml", "sourceFile: escape/pin.toml");
    assert_no_leak(&run(&root, &yaml, &["escape/pin.toml", PAIR[1]]));
}
