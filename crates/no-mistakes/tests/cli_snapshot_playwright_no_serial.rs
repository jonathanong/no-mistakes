use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn playwright_no_serial_json_matches_snapshot() {
    let rule = "playwright-no-serial";
    let strict = rules_path(rule, "configs", "strict.yml");
    let allowlist = rules_path(rule, "configs", "allowlist.yml");
    let cases = [
        (
            "reported",
            rules_path(rule, "fixture", "reported"),
            strict.clone(),
        ),
        (
            "compliant",
            rules_path(rule, "fixture", "compliant"),
            strict,
        ),
        (
            "allowlist",
            rules_path(rule, "fixture", "reported"),
            allowlist,
        ),
    ];
    assert_saved_fixture("playwright_no_serial", &cases);
}
