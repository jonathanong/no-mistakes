use super::*;
use crate::codebase::rules;

type Check = fn(&Path, &NoMistakesConfig, &[PathBuf]) -> Result<Vec<RuleFinding>>;

const CHECKS: &[(&str, Check)] = &[
    (
        "test-no-dependency-pins",
        rules::test_no_dependency_pins::check_with_files,
    ),
    (
        "vitest-test-correspondence",
        rules::vitest_test_correspondence::check_with_files,
    ),
    (
        "github-actions-test-timeout-literals",
        rules::github_actions_test_timeout_literals::check_with_files,
    ),
    ("banned-paths", rules::banned_paths::check_with_files),
    (
        "strict-package-layout",
        rules::strict_package_layout::check_with_files,
    ),
    (
        "csharp-max-lines-per-file",
        rules::csharp_max_lines_per_file::check_with_files,
    ),
    (
        "github-actions-script-expressions",
        rules::github_actions_script_expressions::check_with_files,
    ),
    (
        "banned-renamed-files",
        rules::banned_renamed_files::check_with_files,
    ),
    (
        "require-files-in-subdirs",
        rules::require_files_in_subdirs::check_with_files,
    ),
    (
        "swift-no-raw-print",
        rules::swift_no_raw_print::check_with_files,
    ),
    (
        "required-local-docs",
        rules::required_local_docs::check_with_files,
    ),
    ("doc-consistency", rules::doc_consistency::check_with_files),
    (
        "github-actions-docker-host-ports",
        rules::github_actions_docker_host_ports::check_with_files,
    ),
    (
        "tsconfig-file-coverage",
        rules::tsconfig_file_coverage::check_with_files,
    ),
    (
        "github-actions-composite-step-schema",
        rules::github_actions_composite_step_schema::check_with_files,
    ),
    (
        "version-pin-consistency",
        rules::version_pin_consistency::check_with_files,
    ),
    (
        "lockfile-allowlist",
        rules::lockfile_allowlist::check_with_files,
    ),
    (
        "github-actions-job-permissions",
        rules::github_actions_job_permissions::check_with_files,
    ),
    (
        "package-json-registry-only",
        rules::package_json_registry_only::check_with_files,
    ),
    (
        "csharp-no-async-void-delegate",
        rules::csharp_no_async_void_delegate::check_with_files,
    ),
    (
        "no-empty-or-comments-only-files",
        rules::no_empty_or_comments_only_files::check_with_files,
    ),
    ("no-test-git-sha", rules::no_test_git_sha::check_with_files),
    (
        "require-test-per-subdir",
        rules::require_test_per_subdir::check_with_files,
    ),
    (
        "swift-viewmodel-main-actor",
        rules::swift_viewmodel_main_actor::check_with_files,
    ),
    (
        "github-actions-pinned-hash",
        rules::github_actions_pinned_hash::check_with_files,
    ),
    (
        "file-extension-policy",
        rules::file_extension_policy::check_with_files,
    ),
    (
        "no-sparse-checkout",
        rules::no_sparse_checkout::check_with_files,
    ),
    (
        "package-json-required-fields",
        rules::package_json_required_fields::check_with_files,
    ),
];

#[test]
fn source_entrypoints_reject_malformed_options_and_invalid_include_globs() {
    let mut failures = Vec::new();
    for &(rule, check) in CHECKS {
        let root = crate::codebase::ts_resolver::normalize_path(
            &crate::test_support::rule_fixture_root(rule).join("fail"),
        );
        let original =
            crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
        let files = discover_files(&root, &original.filesystem.skip_directories);
        assert!(!files.is_empty(), "{rule}");
        for malformed_options in [true, false] {
            let mut config = original.clone();
            let selected = config
                .rules
                .iter_mut()
                .find(|item| item.rule == rule)
                .unwrap();
            let expected = if malformed_options {
                selected.options = serde_yaml::Value::Bool(false);
                rule
            } else {
                selected.include = vec!["[".into()];
                "glob"
            };
            match check(&root, &config, &files) {
                Err(error) if format!("{error:#}").contains(expected) => {}
                Err(error) => failures.push(format!(
                    "{rule}: wrong diagnostic for malformed-options={malformed_options}: {error:#}"
                )),
                Ok(findings) => failures.push(format!(
                    "{rule}: accepted malformed-options={malformed_options}; returned {} findings",
                    findings.len()
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
