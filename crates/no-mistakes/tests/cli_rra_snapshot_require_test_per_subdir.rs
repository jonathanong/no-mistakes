use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "require-test-per-subdir";

fn config(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "configs", name)
}

fn fixture(name: &str) -> std::path::PathBuf {
    rules_path(RULE, "fixture", name)
}

#[test]
fn require_test_per_subdir_json_matches_snapshot() {
    assert_saved_fixture(
        "require_test_per_subdir",
        &[
            ("pass", fixture("pass"), config("agents-exclude-shared.yml")),
            ("fail", fixture("fail"), config("agents-exclude-shared.yml")),
            (
                "nested_pass",
                fixture("nested-pass"),
                config("agents-test-mts.yml"),
            ),
            (
                "direct_child_fail",
                fixture("direct-child-fail"),
                config("direct-child.yml"),
            ),
            (
                "direct_child_pass",
                fixture("direct-child-pass"),
                config("direct-child.yml"),
            ),
            (
                "exclude_shared",
                fixture("exclude-shared"),
                config("agents-exclude-shared.yml"),
            ),
            (
                "missing_test",
                fixture("email-source-only"),
                config("agents-test-mts.yml"),
            ),
            (
                "test_present",
                fixture("email-with-test"),
                config("agents-test-mts.yml"),
            ),
            (
                "root_level_file",
                fixture("root-level-file"),
                config("agents-test-mts.yml"),
            ),
            // Omitted `testGlob` uses `*.test.*`, which matches `index.test.ts`.
            (
                "default_glob",
                fixture("default-glob"),
                config("agents-default-glob.yml"),
            ),
            (
                "outside_spec",
                fixture("outside-spec-subroot"),
                config("agents-test-mts.yml"),
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
            // Repository scope still ignores `other/agents` because roots is `app/agents`.
            (
                "app_agents_pass",
                fixture("project-scoped-pass"),
                config("app-agents.yml"),
            ),
            // `.gitkeep` is a direct child of `agents`, not a first-level subdir.
            (
                "root_gitkeep",
                fixture("root-dir-itself"),
                config("agents-test-mts.yml"),
            ),
            (
                "no_agents_tree",
                fixture("outside-file/root"),
                config("agents-test-mts.yml"),
            ),
            // Empty roots skip the fail tree instead of reporting its missing test.
            ("empty_roots", fixture("fail"), config("empty.yml")),
        ],
    );
}
