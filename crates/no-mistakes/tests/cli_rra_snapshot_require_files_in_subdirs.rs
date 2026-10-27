use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "require-files-in-subdirs";

fn config(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "configs", name)
}

fn fixture(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "fixture", name)
}

#[test]
fn require_files_in_subdirs_json_matches_snapshot() {
    assert_saved_fixture(
        "require_files_in_subdirs",
        &[
            ("pass", fixture("pass"), config("queues-required.yml")),
            ("fail", fixture("fail"), config("queues-required.yml")),
            (
                "nested_direct_child_pass",
                fixture("nested-direct-child-pass"),
                config("nested-globs.yml"),
            ),
            (
                "nested_direct_child_fail",
                fixture("nested-direct-child-fail"),
                config("nested-globs.yml"),
            ),
            (
                "bracketed_route",
                fixture("bracketed-route"),
                config("bracketed-route.yml"),
            ),
            (
                "missing_queues_mts",
                fixture("email-package-only"),
                config("package-and-queues.yml"),
            ),
            (
                "required_files_present",
                fixture("email-package-and-queues"),
                config("package-and-queues.yml"),
            ),
            (
                "require_any_of_one",
                fixture("email-enqueues"),
                config("any-enqueues-or-consumers.yml"),
            ),
            (
                "require_any_of_none",
                fixture("email-package-only"),
                config("any-enqueues-or-consumers.yml"),
            ),
            (
                "glob_mts",
                fixture("email-enqueues"),
                config("glob-mts.yml"),
            ),
            (
                "root_level_file",
                fixture("direct-child"),
                config("package-json-only.yml"),
            ),
            (
                "glob_early_match",
                fixture("email-enqueues"),
                config("glob-mts-or-other.yml"),
            ),
            (
                "outside_spec",
                fixture("outside-spec"),
                config("package-json-only.yml"),
            ),
            (
                "glob_falls_through",
                fixture("email-something-mts"),
                config("glob-ts-then-mts.yml"),
            ),
            // `services/../agents` normalizes to `agents` before the subdir walk.
            (
                "parent_segment",
                fixture("parent-segment"),
                config("parent-segment.yml"),
            ),
            (
                "dot_prefix_root",
                fixture("parent-segment"),
                config("dot-agents.yml"),
            ),
            // `.gitkeep` sits directly in `queues`, so it is not a package subdir.
            (
                "spec_root_gitkeep",
                fixture("spec-root-itself"),
                config("package-json-only.yml"),
            ),
            // Empty packages skip the fail tree instead of reporting its missing files.
            ("empty_packages", fixture("fail"), config("empty.yml")),
        ],
    );
}
