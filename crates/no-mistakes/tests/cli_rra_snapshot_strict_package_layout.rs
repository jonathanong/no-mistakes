use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "strict-package-layout";

fn config(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "configs", name)
}

fn fixture(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "fixture", name)
}

#[test]
fn strict_package_layout_json_matches_snapshot() {
    assert_saved_fixture(
        "strict_package_layout",
        &[
            ("pass", fixture("pass"), config("queues-layout.yml")),
            ("fail", fixture("fail"), config("queues-layout.yml")),
            // `other/note.ts` is outside `./packages` and is not a finding.
            ("dot_root", fixture("dot-root"), config("dot-packages.yml")),
            // Empty testFilePatterns keeps the defaults; this tree has no finding.
            (
                "default_test_patterns",
                fixture("default-test-pattern"),
                config("default-patterns.yml"),
            ),
            // Empty packages skip the fail tree instead of flagging its layout.
            ("empty_packages", fixture("fail"), config("empty.yml")),
            // A root that lexically leaves the fixture is skipped, not scanned.
            (
                "escaping_parent",
                fixture("pass"),
                config("escaping-parent.yml"),
            ),
            (
                "escaping_outside",
                fixture("pass"),
                config("escaping-outside.yml"),
            ),
        ],
    );
}
