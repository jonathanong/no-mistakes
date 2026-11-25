use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-no-insta-snapshot-updates";

#[test]
fn github_actions_no_insta_snapshot_updates_json_matches_snapshot() {
    let cases = [
        ("demo", "demo", "enabled"),
        ("scopes", "scopes", "enabled"),
        ("pass", "pass", "enabled"),
        ("unanalyzable", "unanalyzable", "enabled"),
        ("unanalyzable-report", "unanalyzable", "unanalyzable-report"),
        ("suppression", "suppression", "enabled"),
        ("off-by-default", "demo", "disabled"),
    ]
    .map(|(name, fixture, config)| {
        (
            name,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", &format!("{config}.yml")),
        )
    });
    assert_saved_fixture("github_actions_no_insta_snapshot_updates", &cases);
}
