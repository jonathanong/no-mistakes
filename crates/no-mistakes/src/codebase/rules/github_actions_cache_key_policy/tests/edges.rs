use super::super::*;

#[test]
fn incomplete_redirects_leave_the_lockfile_visible() {
    let cases = [
        ("sha256sum widgets.lock &", vec!["sha256sum widgets.lock"]),
        (
            "sha256sum widgets.lock >&",
            vec!["sha256sum widgets.lock >&"],
        ),
        (
            "sha256sum widgets.lock &>",
            vec!["sha256sum widgets.lock &>"],
        ),
        (
            "sha256sum widgets.lock 2>&",
            vec!["sha256sum widgets.lock 2>&"],
        ),
        ("sha256sum widgets.lock >", vec!["sha256sum widgets.lock >"]),
    ];
    for (command, parts) in cases {
        assert_eq!(shell_cmd::parts(command), parts, "{command}");
        assert!(shell_cmd::redirect_targets(command).is_empty(), "{command}");
        assert!(
            text::command_hashes_lockfile(command, "widgets.lock"),
            "{command}"
        );
        let run = format!("digest=$({command})\necho \"digest=${{digest}}\" >\"$GITHUB_OUTPUT\"\n");
        assert_eq!(
            output::proven_output_names(&run, "widgets.lock"),
            vec!["digest".to_string()],
            "{command}"
        );
    }
    assert!(shell_cmd::parts("&").is_empty());
    assert_eq!(shell_cmd::parts(">&"), [">&"]);
    assert_eq!(shell_cmd::parts("&>"), ["&>"]);
    assert_eq!(shell_cmd::parts("2>&"), ["2>&"]);
    assert_eq!(shell_cmd::parts(">"), [">"]);
    for command in ["&", ">&", "&>", "2>&", ">"] {
        assert!(shell_cmd::redirect_targets(command).is_empty(), "{command}");
    }
    assert_eq!(
        shell_cmd::redirect_targets("echo >&1\"widgets.lock\""),
        vec!["1widgets.lock".to_string()]
    );
    assert!(shell_cmd::redirect_targets("echo >&1\"2\"").is_empty());
    assert!(!text::command_hashes_lockfile(
        "sha256sum orders.lock >&1\"widgets.lock\"",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "sha256sum 2>&1 widgets.lock",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "sha256sum widgets.lock 2>out\" orders.lock\"",
        "widgets.lock"
    ));
}

#[test]
fn incomplete_redirect_fixtures_still_prove_the_lockfile() {
    let findings = super::rows("policy", "policy");
    for (file, needle) in [
        ("trailing-amp.yml", "widgets.lock &"),
        ("trailing-fd.yml", "widgets.lock >&"),
        ("end-amp-redirect.yml", "widgets.lock &>"),
        ("end-fd-dup.yml", "widgets.lock 2>&"),
        ("lone-redirect.yml", "widgets.lock >"),
    ] {
        let relative = format!(".github/workflows/{file}");
        let source = super::source("policy", &relative);
        assert!(source.contains(needle), "{file}");
        super::assert_silent(&findings, file);
    }
    let source = super::source("policy", ".github/workflows/digit-suffix.yml");
    assert!(source.contains(">&1\"widgets.lock\""));
    super::assert_needles(
        &findings,
        "digit-suffix.yml",
        &["does not prove a digest of `widgets.lock`"],
    );
}
