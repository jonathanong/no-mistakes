use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-max-inline-script-lines";

#[test]
fn github_actions_max_inline_script_lines_json_matches_snapshot() {
    let cases: Vec<_> = [
        ("boundary", "three"),
        ("suppression", "three"),
        ("default", "default"),
        ("unsupported", "three"),
        ("continuations", "three"),
        ("contexts", "three"),
        ("unreadable", "three"),
        ("no-scripts", "three"),
    ]
    .into_iter()
    .map(|(fixture, config)| {
        (
            fixture,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", &format!("{config}.yml")),
        )
    })
    .collect();
    assert_saved_fixture("github_actions_max_inline_script_lines", &cases);
}

#[test]
fn invalid_max_lines_is_rejected() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(rules_path(RULE, "fixture", "boundary"))
        .arg("--config")
        .arg(rules_path(RULE, "configs", "invalid.yml"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("maxLines must be a positive integer"),
        "{text}"
    );
}

#[test]
fn github_actions_inline_script_scalars_json_matches_snapshot() {
    let cases: Vec<_> = ["quoted-scalars", "block-comments"]
        .into_iter()
        .map(|fixture| {
            (
                fixture,
                rules_path(RULE, "fixture", fixture),
                rules_path(RULE, "configs", "three.yml"),
            )
        })
        .collect();
    assert_saved_fixture("github_actions_inline_script_scalars", &cases);
}

#[test]
fn github_actions_inline_script_followups_json_matches_snapshot() {
    let cases: Vec<_> = ["flow-locations", "block-uses", "anchored-scalars"]
        .into_iter()
        .map(|fixture| {
            (
                fixture,
                rules_path(RULE, "fixture", fixture),
                rules_path(RULE, "configs", "three.yml"),
            )
        })
        .collect();
    assert_saved_fixture("github_actions_inline_script_followups", &cases);
}
