use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

type RuleAcc = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;

use super::{
    agents_md_max_size, banned_renamed_files, doc_consistency, file_extension_policy,
    lockfile_allowlist, no_empty_or_comments_only_files, no_git_identity_mutation,
    package_json_registry_only, require_files_in_subdirs, require_test_per_subdir,
    required_local_docs, rust_max_lines_per_file, rust_no_inline_allows, rust_no_inline_tests,
    shellcheck_runner, strict_package_layout, tsconfig_alias_folder_mapping,
    vitest_test_correspondence,
};
use super::{
    rule_enabled, suppress_rule_findings, RuleFinding, AGENTS_MD_MAX_SIZE, BANNED_RENAMED_FILES,
    DOC_CONSISTENCY, FILE_EXTENSION_POLICY, LOCKFILE_ALLOWLIST, NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    NO_GIT_IDENTITY_MUTATION, PACKAGE_JSON_REGISTRY_ONLY, REQUIRED_DOC_SECTION,
    REQUIRED_LOCAL_DOCS, REQUIRE_FILES_IN_SUBDIRS, REQUIRE_TEST_PER_SUBDIR,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS, SHELLCHECK_RUNNER,
    STRICT_PACKAGE_LAYOUT, TSCONFIG_ALIAS_FOLDER_MAPPING, VITEST_TEST_CORRESPONDENCE,
};

const FILESYSTEM_RULE_IDS: &[&str] = &[
    AGENTS_MD_MAX_SIZE,
    RUST_MAX_LINES_PER_FILE,
    RUST_NO_INLINE_TESTS,
    RUST_NO_INLINE_ALLOWS,
    TSCONFIG_ALIAS_FOLDER_MAPPING,
    NO_GIT_IDENTITY_MUTATION,
    PACKAGE_JSON_REGISTRY_ONLY,
    REQUIRE_TEST_PER_SUBDIR,
    REQUIRE_FILES_IN_SUBDIRS,
    STRICT_PACKAGE_LAYOUT,
    REQUIRED_LOCAL_DOCS,
    REQUIRED_DOC_SECTION,
    NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    VITEST_TEST_CORRESPONDENCE,
    FILE_EXTENSION_POLICY,
    BANNED_RENAMED_FILES,
    LOCKFILE_ALLOWLIST,
    DOC_CONSISTENCY,
    SHELLCHECK_RUNNER,
];

/// Run filesystem rules using a pre-discovered file list so the caller's single
/// `git ls-files` result is reused. Rules run in parallel.
pub fn run_filesystem_rules_with_files(
    root: &Path,
    config_path: Option<&Path>,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    run_filesystem_rules_with_config(root, &config, files)
}

/// Standalone entry point: discover files once, then reuse the with-files
/// dispatcher for every enabled filesystem rule.
pub fn run_filesystem_rules(root: &Path, config_path: Option<&Path>) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    if !FILESYSTEM_RULE_IDS
        .iter()
        .any(|rule_id| rule_enabled(&config, rule_id))
    {
        return Ok(Vec::new());
    }
    let files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    run_filesystem_rules_with_config(root, &config, &files)
}

#[rustfmt::skip]
fn run_filesystem_rules_with_config(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let acc: RuleAcc = Mutex::new(Vec::new());
    rayon::scope(|s| {
        macro_rules! run {
            ($id:expr, $call:expr) => {
                if rule_enabled(config, $id) {
                    s.spawn(|_| { let res = $call; acc.lock().unwrap().push(($id, res)); });
                }
            };
        }
        run!(AGENTS_MD_MAX_SIZE,            agents_md_max_size::check_with_files(root, config, files));
        run!(RUST_MAX_LINES_PER_FILE,       rust_max_lines_per_file::check_with_files(root, config, files));
        run!(RUST_NO_INLINE_TESTS,          rust_no_inline_tests::check_with_files(root, config, files));
        run!(RUST_NO_INLINE_ALLOWS,         rust_no_inline_allows::check_with_files(root, config, files));
        run!(TSCONFIG_ALIAS_FOLDER_MAPPING, tsconfig_alias_folder_mapping::check_with_files(root, config, files));
        run!(NO_GIT_IDENTITY_MUTATION,      no_git_identity_mutation::check_with_files(root, config, files));
        run!(PACKAGE_JSON_REGISTRY_ONLY,    package_json_registry_only::check_with_files(root, config, files));
        run!(REQUIRE_TEST_PER_SUBDIR,       require_test_per_subdir::check_with_files(root, config, files));
        run!(REQUIRE_FILES_IN_SUBDIRS,      require_files_in_subdirs::check_with_files(root, config, files));
        run!(STRICT_PACKAGE_LAYOUT,         strict_package_layout::check_with_files(root, config, files));
        run!(REQUIRED_LOCAL_DOCS,           required_local_docs::check_with_files(root, config, files));
        run!(REQUIRED_DOC_SECTION,          required_local_docs::check_required_doc_section_with_files(root, config, files));
        run!(NO_EMPTY_OR_COMMENTS_ONLY_FILES, no_empty_or_comments_only_files::check_with_files(root, config, files));
        run!(VITEST_TEST_CORRESPONDENCE,    vitest_test_correspondence::check_with_files(root, config, files));
        run!(FILE_EXTENSION_POLICY,         file_extension_policy::check_with_files(root, config, files));
        run!(BANNED_RENAMED_FILES,          banned_renamed_files::check_with_files(root, config, files));
        run!(LOCKFILE_ALLOWLIST,            lockfile_allowlist::check_with_files(root, config, files));
        run!(DOC_CONSISTENCY,               doc_consistency::check_with_files(root, config, files));
        run!(SHELLCHECK_RUNNER,             shellcheck_runner::check_with_files(root, config, files));
    });
    let mut results = acc.into_inner().unwrap();
    results.sort_unstable_by_key(|(id, _)| *id);
    let mut findings = Vec::new();
    for (_, r) in results { findings.extend(r?); }
    suppress_rule_findings(root, &mut findings);
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
