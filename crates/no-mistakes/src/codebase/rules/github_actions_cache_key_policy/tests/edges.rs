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

fn proven(script: &str) -> Vec<String> {
    let run = format!("digest=$(sha256sum widgets.lock)\n{script}\n");
    output::proven_output_names(&run, "widgets.lock")
}

#[test]
fn group_redirect_applies_to_commands_that_do_not_redirect_stdout() {
    let digest = vec!["digest".to_string()];
    for script in [
        "{ echo \"digest=$digest\"; } >> \"$GITHUB_OUTPUT\"",
        "( echo \"digest=$digest\" ) >> \"$GITHUB_OUTPUT\"",
        "{ echo \"digest=$digest\"; }>>\"$GITHUB_OUTPUT\"",
        "(echo \"digest=$digest\")>>\"$GITHUB_OUTPUT\"",
        "{ echo \"digest=$digest\" 2>&1; } >> \"$GITHUB_OUTPUT\"",
        "{ echo \"digest=$digest\" 2>/tmp/orders; } >> \"$GITHUB_OUTPUT\"",
        "{ echo \"}; }\"; echo \"digest=$digest\"; } >> \"$GITHUB_OUTPUT\"",
        "{ echo '}; }'; echo \"digest=$digest\"; } >> \"$GITHUB_OUTPUT\"",
        "(\n  echo \"digest=$digest\"\n) >> \"$GITHUB_OUTPUT\"",
        "{\n  digest=$(sha256sum widgets.lock)\n  echo \"digest=$digest\"\n} >> \"$GITHUB_OUTPUT\"",
    ] {
        assert_eq!(proven(script), digest, "{script}");
    }
    assert_eq!(
        output::proven_output_names(
            "{ digest=$(sha256sum widgets.lock); echo \"digest=$digest\"; } >> \"$GITHUB_OUTPUT\"\n",
            "widgets.lock",
        ),
        digest
    );
    for script in [
        "{ echo \"digest=$digest\"; }",
        "( echo \"digest=$digest\" )",
        "{ echo \"digest=$digest\"; } >> /tmp/orders",
        "( echo \"digest=$digest\" ) >> /tmp/orders",
        "{ echo \"digest=$digest\"; } >> '$GITHUB_OUTPUT'",
        "{ echo \"digest=$digest\"; } 2>&1",
        "{ echo \"digest=$digest\" > /tmp/orders; } >> \"$GITHUB_OUTPUT\"",
        "{ echo \"digest=$digest\" >&2; } >> \"$GITHUB_OUTPUT\"",
        "{echo \"digest=$digest\";} >> \"$GITHUB_OUTPUT\"",
        "{ echo \"digest=$digest\" | cat /tmp/orders; } >> \"$GITHUB_OUTPUT\"",
    ] {
        assert!(proven(script).is_empty(), "{script}");
    }
    assert_eq!(
        proven(
            "{ echo \"skip=1\"; } >> /tmp/orders\necho \"digest=$digest\" >> \"$GITHUB_OUTPUT\""
        ),
        digest
    );
}

#[test]
fn group_redirect_fixtures_match_the_suffix() {
    let findings = super::rows("policy", "policy");
    for file in ["group-output.yml", "subshell-output.yml"] {
        let source = super::source("policy", &format!(".github/workflows/{file}"));
        assert!(source.contains(">> \"$GITHUB_OUTPUT\""), "{file}");
        super::assert_silent(&findings, file);
    }
    for file in ["group-plain.yml", "group-other-file.yml"] {
        let source = super::source("policy", &format!(".github/workflows/{file}"));
        assert!(source.contains("echo \"digest=$digest\""), "{file}");
        super::assert_needles(
            &findings,
            file,
            &["does not prove a digest of `widgets.lock`"],
        );
    }
}
