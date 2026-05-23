use anyhow::Result;
use std::path::{Path, PathBuf};

use super::{
    agents_md_max_size, banned_renamed_files, doc_consistency, file_extension_policy,
    lockfile_allowlist, no_empty_or_comments_only_files, no_git_identity_mutation,
    package_json_registry_only, require_files_in_subdirs, require_test_per_subdir,
    required_local_docs, rust_max_lines_per_file, rust_no_inline_allows, rust_no_inline_tests,
    shellcheck_runner, strict_package_layout, tsconfig_alias_folder_mapping,
    vitest_test_correspondence,
};
use super::{
    rule_enabled, RuleFinding, AGENTS_MD_MAX_SIZE, BANNED_RENAMED_FILES, DOC_CONSISTENCY,
    FILE_EXTENSION_POLICY, LOCKFILE_ALLOWLIST, NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    NO_GIT_IDENTITY_MUTATION, PACKAGE_JSON_REGISTRY_ONLY, REQUIRED_DOC_SECTION,
    REQUIRED_LOCAL_DOCS, REQUIRE_FILES_IN_SUBDIRS, REQUIRE_TEST_PER_SUBDIR,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS, SHELLCHECK_RUNNER,
    STRICT_PACKAGE_LAYOUT, TSCONFIG_ALIAS_FOLDER_MAPPING, VITEST_TEST_CORRESPONDENCE,
};

/// Run filesystem rules using a pre-discovered file list so the caller's single
/// `git ls-files` result is reused — no second walk.
pub fn run_filesystem_rules_with_files(
    root: &Path,
    config_path: Option<&Path>,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    let mut findings = Vec::new();
    if rule_enabled(&config, AGENTS_MD_MAX_SIZE) {
        findings.extend(agents_md_max_size::check_with_files(root, &config, files)?);
    }
    if rule_enabled(&config, RUST_MAX_LINES_PER_FILE) {
        findings.extend(rust_max_lines_per_file::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, RUST_NO_INLINE_TESTS) {
        findings.extend(rust_no_inline_tests::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, RUST_NO_INLINE_ALLOWS) {
        findings.extend(rust_no_inline_allows::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, TSCONFIG_ALIAS_FOLDER_MAPPING) {
        findings.extend(tsconfig_alias_folder_mapping::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, NO_GIT_IDENTITY_MUTATION) {
        findings.extend(no_git_identity_mutation::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, PACKAGE_JSON_REGISTRY_ONLY) {
        findings.extend(package_json_registry_only::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, REQUIRE_TEST_PER_SUBDIR) {
        findings.extend(require_test_per_subdir::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, REQUIRE_FILES_IN_SUBDIRS) {
        findings.extend(require_files_in_subdirs::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, STRICT_PACKAGE_LAYOUT) {
        findings.extend(strict_package_layout::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, REQUIRED_LOCAL_DOCS) {
        findings.extend(required_local_docs::check_with_files(root, &config, files)?);
    }
    if rule_enabled(&config, REQUIRED_DOC_SECTION) {
        findings.extend(required_local_docs::check_required_doc_section_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, NO_EMPTY_OR_COMMENTS_ONLY_FILES) {
        findings.extend(no_empty_or_comments_only_files::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, VITEST_TEST_CORRESPONDENCE) {
        findings.extend(vitest_test_correspondence::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, FILE_EXTENSION_POLICY) {
        findings.extend(file_extension_policy::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, BANNED_RENAMED_FILES) {
        findings.extend(banned_renamed_files::check_with_files(
            root, &config, files,
        )?);
    }
    if rule_enabled(&config, LOCKFILE_ALLOWLIST) {
        findings.extend(lockfile_allowlist::check_with_files(root, &config, files)?);
    }
    if rule_enabled(&config, DOC_CONSISTENCY) {
        findings.extend(doc_consistency::check_with_files(root, &config, files)?);
    }
    if rule_enabled(&config, SHELLCHECK_RUNNER) {
        findings.extend(shellcheck_runner::check_with_files(root, &config, files)?);
    }
    Ok(findings)
}

/// Standalone entry point — each rule does its own discovery.
pub fn run_filesystem_rules(root: &Path, config_path: Option<&Path>) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    let mut findings = Vec::new();
    if rule_enabled(&config, AGENTS_MD_MAX_SIZE) {
        findings.extend(agents_md_max_size::check(root, &config)?);
    }
    if rule_enabled(&config, RUST_MAX_LINES_PER_FILE) {
        findings.extend(rust_max_lines_per_file::check(root, &config)?);
    }
    if rule_enabled(&config, RUST_NO_INLINE_TESTS) {
        findings.extend(rust_no_inline_tests::check(root, &config)?);
    }
    if rule_enabled(&config, RUST_NO_INLINE_ALLOWS) {
        findings.extend(rust_no_inline_allows::check(root, &config)?);
    }
    if rule_enabled(&config, TSCONFIG_ALIAS_FOLDER_MAPPING) {
        findings.extend(tsconfig_alias_folder_mapping::check(root, &config)?);
    }
    if rule_enabled(&config, NO_GIT_IDENTITY_MUTATION) {
        findings.extend(no_git_identity_mutation::check(root, &config)?);
    }
    if rule_enabled(&config, PACKAGE_JSON_REGISTRY_ONLY) {
        findings.extend(package_json_registry_only::check(root, &config)?);
    }
    if rule_enabled(&config, REQUIRE_TEST_PER_SUBDIR) {
        findings.extend(require_test_per_subdir::check(root, &config)?);
    }
    if rule_enabled(&config, REQUIRE_FILES_IN_SUBDIRS) {
        findings.extend(require_files_in_subdirs::check(root, &config)?);
    }
    if rule_enabled(&config, STRICT_PACKAGE_LAYOUT) {
        findings.extend(strict_package_layout::check(root, &config)?);
    }
    if rule_enabled(&config, REQUIRED_LOCAL_DOCS) {
        findings.extend(required_local_docs::check(root, &config)?);
    }
    if rule_enabled(&config, REQUIRED_DOC_SECTION) {
        findings.extend(required_local_docs::check_required_doc_section(
            root, &config,
        )?);
    }
    if rule_enabled(&config, NO_EMPTY_OR_COMMENTS_ONLY_FILES) {
        findings.extend(no_empty_or_comments_only_files::check(root, &config)?);
    }
    if rule_enabled(&config, VITEST_TEST_CORRESPONDENCE) {
        findings.extend(vitest_test_correspondence::check(root, &config)?);
    }
    if rule_enabled(&config, FILE_EXTENSION_POLICY) {
        findings.extend(file_extension_policy::check(root, &config)?);
    }
    if rule_enabled(&config, BANNED_RENAMED_FILES) {
        findings.extend(banned_renamed_files::check(root, &config)?);
    }
    if rule_enabled(&config, LOCKFILE_ALLOWLIST) {
        findings.extend(lockfile_allowlist::check(root, &config)?);
    }
    if rule_enabled(&config, DOC_CONSISTENCY) {
        findings.extend(doc_consistency::check(root, &config)?);
    }
    if rule_enabled(&config, SHELLCHECK_RUNNER) {
        findings.extend(shellcheck_runner::check(root, &config)?);
    }
    Ok(findings)
}
