macro_rules! filesystem_rules {
    ($macro:ident) => {
        $macro! {
            AGENTS_MD_MAX_SIZE => agents_md_max_size::check_with_files,
            GITHUB_ACTIONS_PINNED_HASH => github_actions_pinned_hash::check_with_files,
            CONFIG_PATH_REFERENCES => config_path_references::check_with_files,
            FINITE_SET_CONSISTENCY => finite_set_consistency::check_with_files,
            FORBIDDEN_WORKSPACE_CLOSURE => forbidden_workspace_closure::check_with_files,
            STRUCTURED_CONFIG_POLICY => structured_config_policy::check_with_files,
            TSCONFIG_ALIAS_FOLDER_MAPPING => tsconfig_alias_folder_mapping::check_with_files,
            NO_GIT_IDENTITY_MUTATION => no_git_identity_mutation::check_with_files,
            PACKAGE_JSON_REGISTRY_ONLY => package_json_registry_only::check_with_files,
            PACKAGE_JSON_WORKSPACE_COVERAGE => package_json_workspace_coverage::check_with_files,
            PRODUCTION_DEPENDENCY_DECLARATIONS => production_dependency_declarations::check_with_files,
            WORKSPACE_PACKAGE_CYCLES => workspace_package_cycles::check_with_files,
            REQUIRED_COMPANION_IMPORTS => required_companion_imports::check_with_files,
            REQUIRE_TEST_PER_SUBDIR => require_test_per_subdir::check_with_files,
            REQUIRE_FILES_IN_SUBDIRS => require_files_in_subdirs::check_with_files,
            STRICT_PACKAGE_LAYOUT => strict_package_layout::check_with_files,
            REQUIRED_LOCAL_DOCS => required_local_docs::check_with_files,
            REQUIRED_DOC_SECTION => required_local_docs::check_required_doc_section_with_files,
            NO_EMPTY_OR_COMMENTS_ONLY_FILES => no_empty_or_comments_only_files::check_with_files,
            VITEST_TEST_CORRESPONDENCE => vitest_test_correspondence::check_with_files,
            FILE_EXTENSION_POLICY => file_extension_policy::check_with_files,
            BANNED_PATHS => banned_paths::check_with_files,
            BANNED_RENAMED_FILES => banned_renamed_files::check_with_files,
            INTEGRATION_TEST_NO_MOCKS => integration_test_no_mocks::check_with_files,
            TEST_EMAIL_DOMAIN_POLICY => test_email_domain_policy::check_with_files,
            MARKDOWN_LINK_DISPLAY_TEXT => markdown_link_display_text::check_with_files,
            LOCKFILE_ALLOWLIST => lockfile_allowlist::check_with_files,
            DOC_CONSISTENCY => doc_consistency::check_with_files,
            SHELLCHECK_RUNNER => shellcheck_runner::check_with_files,
        }
    };
}

pub(super) fn rust_rules_enabled(config: &crate::config::v2::NoMistakesConfig) -> bool {
    super::rule_enabled(config, super::RUST_MAX_LINES_PER_FILE)
        || super::rule_enabled(config, super::RUST_NO_INLINE_TESTS)
        || super::rule_enabled(config, super::RUST_NO_INLINE_ALLOWS)
}
