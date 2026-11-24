use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn dependabot_cooldown_policy_json_matches_snapshot() {
    let rule = "dependabot-cooldown-policy";
    let config = rules_path(rule, "configs", "policy.yml");
    let cases = [
        "compliant",
        "violations",
        "malformed",
        "both-paths",
        "anchors",
        "chained-merges",
        "sequence-merges",
        "low-explicit",
        "incomplete-merge",
        "incomplete-sequence-merges",
        "incomplete-tagged-merges",
    ]
    .map(|name| (name, rules_path(rule, "fixture", name), config.clone()));
    assert_saved_fixture("dependabot_cooldown_policy", &cases);
}
