use super::*;
use crate::codebase::rules;

type Standalone = fn(&Path, &NoMistakesConfig) -> Result<Vec<RuleFinding>>;
type Prepared = fn(&Path, &NoMistakesConfig, &[PathBuf]) -> Result<Vec<RuleFinding>>;

#[test]
fn filesystem_entrypoints_match_prepared_inventory_on_saved_positive_and_negative_cases() {
    let cases: &[(&str, &str, &str, bool, Standalone, Prepared)] = &[
        (
            "file-extension-policy",
            "pass",
            "backend-ts.yml",
            false,
            rules::file_extension_policy::check,
            rules::file_extension_policy::check_with_files,
        ),
        (
            "file-extension-policy",
            "fail",
            "backend-js.yml",
            true,
            rules::file_extension_policy::check,
            rules::file_extension_policy::check_with_files,
        ),
        (
            "banned-renamed-files",
            "pass",
            "middleware-web.yml",
            false,
            rules::banned_renamed_files::check,
            rules::banned_renamed_files::check_with_files,
        ),
        (
            "banned-renamed-files",
            "fail",
            "middleware-web.yml",
            true,
            rules::banned_renamed_files::check,
            rules::banned_renamed_files::check_with_files,
        ),
        (
            "require-test-per-subdir",
            "email-with-test",
            "agents-test-mts.yml",
            false,
            rules::require_test_per_subdir::check,
            rules::require_test_per_subdir::check_with_files,
        ),
        (
            "require-test-per-subdir",
            "email-source-only",
            "agents-test-mts.yml",
            true,
            rules::require_test_per_subdir::check,
            rules::require_test_per_subdir::check_with_files,
        ),
        (
            "require-files-in-subdirs",
            "pass",
            "queues-required.yml",
            false,
            rules::require_files_in_subdirs::check,
            rules::require_files_in_subdirs::check_with_files,
        ),
        (
            "require-files-in-subdirs",
            "fail",
            "queues-required.yml",
            true,
            rules::require_files_in_subdirs::check,
            rules::require_files_in_subdirs::check_with_files,
        ),
        (
            "agents-md-max-size",
            "pass",
            "pass.yml",
            false,
            rules::agents_md_max_size::check,
            rules::agents_md_max_size::check_with_files,
        ),
        (
            "agents-md-max-size",
            "fail",
            "fail.yml",
            true,
            rules::agents_md_max_size::check,
            rules::agents_md_max_size::check_with_files,
        ),
    ];
    for &(rule, fixture, config_file, fails, standalone, prepared) in cases {
        let fixtures = crate::test_support::rule_fixture_root(rule);
        let root = crate::codebase::ts_resolver::normalize_path(&fixtures.join(fixture));
        let config = crate::config::v2::load_v2_config(
            &root,
            Some(&fixtures.parent().unwrap().join("configs").join(config_file)),
        )
        .unwrap();
        let files = discover_files(&root, &config.filesystem.skip_directories);
        assert!(!files.is_empty(), "{rule}/{fixture} inventory");
        let actual = standalone(&root, &config).unwrap();
        assert_eq!(!actual.is_empty(), fails, "{rule}/{fixture}: {actual:?}");
        let expected = prepared(&root, &config, &files).unwrap();
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "{rule}/{fixture}"
        );
    }
}
