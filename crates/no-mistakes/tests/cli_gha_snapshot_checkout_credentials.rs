use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-checkout-credentials";

#[test]
fn github_actions_checkout_credentials_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_checkout_credentials",
        &[
            (
                "pass",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "default.yml"),
            ),
            (
                "fail",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "default.yml"),
            ),
            (
                "flow",
                rules_path(RULE, "fixture", "flow"),
                rules_path(RULE, "configs", "default.yml"),
            ),
            (
                "suppression",
                rules_path(RULE, "fixture", "suppression"),
                rules_path(RULE, "configs", "default.yml"),
            ),
            (
                "abstain",
                rules_path(RULE, "fixture", "abstain"),
                rules_path(RULE, "configs", "ignore.yml"),
            ),
            (
                "assume-off",
                rules_path(RULE, "fixture", "assume-unspecified"),
                rules_path(RULE, "configs", "ignore.yml"),
            ),
            (
                "assume-on",
                rules_path(RULE, "fixture", "assume-unspecified"),
                rules_path(RULE, "configs", "assume-on.yml"),
            ),
            (
                "permission-bounds",
                rules_path(RULE, "fixture", "permission-bounds"),
                rules_path(RULE, "configs", "default.yml"),
            ),
            (
                "permission-bounds-max-advise",
                rules_path(RULE, "fixture", "permission-bounds"),
                rules_path(RULE, "configs", "max-advise.yml"),
            ),
            (
                "permission-bounds-max-ignore",
                rules_path(RULE, "fixture", "permission-bounds"),
                rules_path(RULE, "configs", "max-ignore.yml"),
            ),
            (
                "permission-bounds-low-report",
                rules_path(RULE, "fixture", "permission-bounds"),
                rules_path(RULE, "configs", "low-report.yml"),
            ),
            (
                "permission-bounds-low-advise",
                rules_path(RULE, "fixture", "permission-bounds"),
                rules_path(RULE, "configs", "low-advise.yml"),
            ),
            (
                "permission-bounds-low-ignore",
                rules_path(RULE, "fixture", "permission-bounds"),
                rules_path(RULE, "configs", "low-ignore.yml"),
            ),
            (
                "unanalyzable-max-report",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "max-report.yml"),
            ),
            (
                "unanalyzable-max-advise",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "max-advise.yml"),
            ),
            (
                "unanalyzable-max-ignore",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "max-ignore.yml"),
            ),
            (
                "unanalyzable-low-report",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "low-report.yml"),
            ),
            (
                "unanalyzable-low-advise",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "low-advise.yml"),
            ),
            (
                "unanalyzable-low-ignore",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "low-ignore.yml"),
            ),
            (
                "ref-safety",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "default.yml"),
            ),
            (
                "ref-safety-max-advise",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "max-advise.yml"),
            ),
            (
                "ref-safety-max-ignore",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "max-ignore.yml"),
            ),
            (
                "ref-safety-low-report",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "low-report.yml"),
            ),
            (
                "ref-safety-low-advise",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "low-advise.yml"),
            ),
            (
                "ref-safety-low-ignore",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "low-ignore.yml"),
            ),
            (
                "ref-safety-configured-report",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-report.yml"),
            ),
            (
                "ref-safety-configured-advise",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-advise.yml"),
            ),
            (
                "ref-safety-configured-ignore",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-ignore.yml"),
            ),
            (
                "ref-safety-configured-low-report",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-low-report.yml"),
            ),
            (
                "ref-safety-configured-low-advise",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-low-advise.yml"),
            ),
            (
                "ref-safety-configured-low-ignore",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-low-ignore.yml"),
            ),
            (
                "ref-safety-configured-heads",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "configured-heads.yml"),
            ),
            (
                "ref-safety-safe",
                rules_path(RULE, "fixture", "ref-safety"),
                rules_path(RULE, "configs", "ref-safe.yml"),
            ),
            (
                "hex40-report",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "hex40-report.yml"),
            ),
            (
                "hex40-advise",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "hex40-advise.yml"),
            ),
            (
                "hex40-ignore",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "hex40-ignore.yml"),
            ),
            (
                "hex64-report",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "hex64-report.yml"),
            ),
            (
                "hex64-advise",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "hex64-advise.yml"),
            ),
            (
                "hex64-ignore",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "hex64-ignore.yml"),
            ),
            (
                "nonhex40-report",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "nonhex40-report.yml"),
            ),
            (
                "nonhex40-advise",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "nonhex40-advise.yml"),
            ),
            (
                "nonhex40-ignore",
                rules_path(RULE, "fixture", "ref-hex-default"),
                rules_path(RULE, "configs", "nonhex40-ignore.yml"),
            ),
        ],
    );
}
