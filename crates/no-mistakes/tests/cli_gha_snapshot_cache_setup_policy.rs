use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "github-actions-cache-setup-policy";

#[test]
fn github_actions_cache_setup_policy_matches_snapshot() {
    let cases = [
        "report",
        "advise",
        "ignore",
        "cache-only",
        "setup-only",
        "external-action",
    ]
    .map(|mode| {
        (
            mode,
            rules_path(RULE, "fixture", "workflows"),
            rules_path(RULE, "configs", &format!("{mode}.yml")),
        )
    });
    let result = assert_saved_fixture_report("github_actions_cache_setup_policy", &cases, &[], &[]);
    for mode in ["report", "advise", "ignore"] {
        assert_eq!(result[mode]["exitCode"], 1);
        assert!(result[mode]["report"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["target"] == "setup-deadline"));
    }
    assert!(!result["advise"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(result["ignore"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
    let external = &result["external-action"];
    assert_eq!(external["exitCode"], 1);
    let rows = external["report"]["rules"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["target"], "cache-key-shape");
    assert_eq!(rows[0]["file"], "tools/actions/example/action.yml");
    for (mode, expected) in [
        ("cache-only", "cache-key-shape"),
        ("setup-only", "setup-deadline"),
    ] {
        let rows = result[mode]["report"]["rules"].as_array().unwrap();
        assert!(!rows.is_empty(), "{mode}");
        assert!(rows
            .iter()
            .all(|row| row["target"] == expected && row["file"] == ".github/workflows/steps.yml"));
    }
}

#[test]
fn github_actions_cache_setup_policy_rejects_invalid_options_before_checking() {
    for name in [
        "missing",
        "bad-regex",
        "bad-cap",
        "bad-include",
        "duplicate-cache",
        "duplicate-setup",
        "unknown-option",
        "bad-family",
        "bad-path",
    ] {
        let root = rules_path(RULE, "fixture", "workflows");
        let config = rules_path(RULE, "configs", &format!("{name}.yml"));
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["--lock-timeout", "0", "check", "--root"])
            .arg(root)
            .arg("--config")
            .arg(config)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{name}: {:?}", output);
        assert!(output.stdout.is_empty(), "{name}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(RULE),
            "{name}: {:?}",
            output
        );
    }
}
