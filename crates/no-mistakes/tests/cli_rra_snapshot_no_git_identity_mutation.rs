use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "no-git-identity-mutation";

#[test]
fn no_git_identity_mutation_json_matches_snapshot() {
    let empty = rules_path(RULE, "configs", "empty.yml");
    let exclude_scripts = rules_path(RULE, "configs", "exclude-scripts.yml");
    let conditional = rules_path(RULE, "configs", "conditional-github-workflows.yml");
    assert_saved_fixture(
        "no_git_identity_mutation",
        &[
            ("pass", rules_path(RULE, "fixture", "pass"), empty.clone()),
            ("fail", rules_path(RULE, "fixture", "fail"), empty.clone()),
            (
                "readonly_lookup",
                rules_path(RULE, "fixture", "readonly-pass"),
                empty.clone(),
            ),
            (
                "shell_form",
                rules_path(RULE, "fixture/cases", "shell-form"),
                empty.clone(),
            ),
            (
                "env_var_form",
                rules_path(RULE, "fixture/cases", "env-var-form"),
                empty.clone(),
            ),
            (
                "git_config_email",
                rules_path(RULE, "fixture/cases", "git-config-email"),
                empty.clone(),
            ),
            (
                "read_only_shell",
                rules_path(RULE, "fixture/cases", "read-only-shell"),
                empty.clone(),
            ),
            (
                "array_form",
                rules_path(RULE, "fixture/cases", "array-form"),
                empty.clone(),
            ),
            (
                "read_only_array",
                rules_path(RULE, "fixture/cases", "read-only-array"),
                empty.clone(),
            ),
            (
                "helper_form",
                rules_path(RULE, "fixture/cases", "helper-form"),
                empty.clone(),
            ),
            (
                "read_only_helper",
                rules_path(RULE, "fixture/cases", "read-only-helper"),
                empty.clone(),
            ),
            (
                "excluded_path",
                rules_path(RULE, "fixture/cases", "excluded-path"),
                exclude_scripts.clone(),
            ),
            (
                "exclude_paths",
                rules_path(RULE, "fixture/cases", "exclude-paths"),
                exclude_scripts,
            ),
            (
                "managed_runners",
                rules_path(RULE, "fixture/cases", "managed-runners-only"),
                conditional.clone(),
            ),
            (
                "self_hosted",
                rules_path(RULE, "fixture/cases", "self-hosted-runner"),
                conditional.clone(),
            ),
            (
                "quoted_managed",
                rules_path(RULE, "fixture/cases", "quoted-managed-runner"),
                conditional.clone(),
            ),
            (
                "mapping_managed",
                rules_path(RULE, "fixture", "mapping-managed-pass"),
                conditional,
            ),
            (
                "line_number",
                rules_path(RULE, "fixture/cases", "line-number"),
                empty.clone(),
            ),
            (
                "non_script",
                rules_path(RULE, "fixture/cases", "non-script"),
                empty.clone(),
            ),
            (
                "commented_line",
                rules_path(RULE, "fixture/cases", "commented-line"),
                empty.clone(),
            ),
            (
                "echo_line",
                rules_path(RULE, "fixture/cases", "echo-line"),
                empty.clone(),
            ),
            (
                "shebang_deploy",
                rules_path(RULE, "fixture/cases", "shebang-deploy"),
                empty.clone(),
            ),
            (
                "target_roots",
                rules_path(RULE, "fixture/cases", "target-roots"),
                empty.clone(),
            ),
            (
                "line_continued",
                rules_path(RULE, "fixture/cases", "line-continued"),
                empty.clone(),
            ),
            // Whole directory: mutations.sh lines plus managed.yml and self-hosted.yml.
            (
                "literal_prefixes",
                rules_path(RULE, "fixture", "literal-prefixes"),
                empty,
            ),
            (
                "literal_prefixes_managed",
                rules_path(RULE, "fixture", "literal-prefixes"),
                rules_path(RULE, "configs", "conditional-star-yml.yml"),
            ),
            (
                "literal_prefixes_excluded",
                rules_path(RULE, "fixture", "literal-prefixes"),
                rules_path(RULE, "configs", "exclude-mutations.yml"),
            ),
        ],
    );
}
