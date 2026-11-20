use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-cache-key-policy";

#[test]
fn github_actions_cache_key_policy_json_matches_snapshot() {
    let cases: Vec<_> = ["policy", "restore-ok", "hash-off", "unreadable"]
        .into_iter()
        .map(|fixture| {
            let config = if fixture == "unreadable" {
                "policy"
            } else {
                fixture
            };
            (
                fixture,
                rules_path(RULE, "fixture", fixture),
                rules_path(RULE, "configs", &format!("{config}.yml")),
            )
        })
        .collect();
    assert_saved_fixture("github_actions_cache_key_policy", &cases);
}

#[test]
fn empty_cache_list_is_rejected() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(rules_path(RULE, "fixture", "policy"))
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
        text.contains("caches must list at least one cache"),
        "{text}"
    );
}
