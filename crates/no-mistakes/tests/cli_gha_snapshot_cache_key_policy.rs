use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-cache-key-policy";

#[test]
fn github_actions_cache_key_policy_json_matches_snapshot() {
    let mut cases: Vec<_> = ["policy", "restore-ok", "hash-off", "unreadable"]
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
    let invalid = rules_path("options-audit", "fixture", "issue-3958");
    cases.push((
        "invalid_forbidden_path",
        invalid.clone(),
        invalid.join("cache.yml"),
    ));
    assert_saved_fixture("github_actions_cache_key_policy", &cases);
}

#[test]
fn github_actions_cache_broad_hash_files_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "broad-hash");
    assert_saved_fixture(
        "github_actions_cache_broad_hash_files",
        &[
            (
                "enabled",
                root.clone(),
                rules_path(RULE, "configs", "broad-hash.yml"),
            ),
            (
                "default",
                root.clone(),
                rules_path(RULE, "configs", "restore-ok.yml"),
            ),
            (
                "restore-rejected",
                root,
                rules_path(RULE, "configs", "policy.yml"),
            ),
        ],
    );
}

#[test]
fn malformed_hash_files_preserve_only_findings_from_complete_expressions() {
    assert_saved_fixture(
        "github_actions_cache_malformed_hash_files",
        &[(
            "malformed",
            rules_path(RULE, "fixture", "malformed-broad-hash"),
            rules_path(RULE, "configs", "broad-hash.yml"),
        )],
    );
}

#[test]
fn github_actions_cache_forbidden_paths_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "forbidden-paths");
    assert_saved_fixture(
        "github_actions_cache_forbidden_paths",
        &[
            (
                "enabled",
                root.clone(),
                rules_path(RULE, "configs", "forbidden-paths.yml"),
            ),
            (
                "custom-message",
                root.clone(),
                rules_path(RULE, "configs", "forbidden-paths-custom-message.yml"),
            ),
            (
                "rule-message",
                root.clone(),
                rules_path(RULE, "configs", "forbidden-paths-rule-message.yml"),
            ),
            ("unset", root, rules_path(RULE, "configs", "policy.yml")),
        ],
    );
}

#[test]
fn empty_cache_list_is_rejected() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(rules_path(RULE, "fixture", "policy"))
        .arg("--config")
        .arg(rules_path(RULE, "configs", "invalid.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["incomplete"][0]["consumer"], RULE);
    assert_eq!(report["incomplete"][0]["property"], "options.caches");
}
