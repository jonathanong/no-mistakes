use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

type RuleAcc = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;

use super::{
    agents_md_max_size, banned_renamed_files, doc_consistency, file_extension_policy,
    github_actions_pinned_hash, lockfile_allowlist, no_empty_or_comments_only_files,
    no_git_identity_mutation, package_json_registry_only, require_files_in_subdirs,
    require_test_per_subdir, required_local_docs, rust_rules_combined, shellcheck_runner,
    strict_package_layout, tsconfig_alias_folder_mapping, vitest_test_correspondence,
};

mod preserved;
use super::{
    rule_enabled, suppress_rule_findings, RuleFinding, AGENTS_MD_MAX_SIZE, BANNED_RENAMED_FILES,
    DOC_CONSISTENCY, FILE_EXTENSION_POLICY, LOCKFILE_ALLOWLIST, NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    NO_GIT_IDENTITY_MUTATION, PACKAGE_JSON_REGISTRY_ONLY, REQUIRED_DOC_SECTION,
    REQUIRED_LOCAL_DOCS, REQUIRE_FILES_IN_SUBDIRS, REQUIRE_TEST_PER_SUBDIR,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS, SHELLCHECK_RUNNER,
    STRICT_PACKAGE_LAYOUT, TSCONFIG_ALIAS_FOLDER_MAPPING, VITEST_TEST_CORRESPONDENCE,
};
const GITHUB_ACTIONS_PINNED_HASH: &str = github_actions_pinned_hash::RULE_ID;

macro_rules! filesystem_rules {
    ($macro:ident) => {
        $macro! {
            AGENTS_MD_MAX_SIZE => agents_md_max_size::check_with_files,
            GITHUB_ACTIONS_PINNED_HASH => github_actions_pinned_hash::check_with_files,
            TSCONFIG_ALIAS_FOLDER_MAPPING => tsconfig_alias_folder_mapping::check_with_files,
            NO_GIT_IDENTITY_MUTATION => no_git_identity_mutation::check_with_files,
            PACKAGE_JSON_REGISTRY_ONLY => package_json_registry_only::check_with_files,
            REQUIRE_TEST_PER_SUBDIR => require_test_per_subdir::check_with_files,
            REQUIRE_FILES_IN_SUBDIRS => require_files_in_subdirs::check_with_files,
            STRICT_PACKAGE_LAYOUT => strict_package_layout::check_with_files,
            REQUIRED_LOCAL_DOCS => required_local_docs::check_with_files,
            REQUIRED_DOC_SECTION => required_local_docs::check_required_doc_section_with_files,
            NO_EMPTY_OR_COMMENTS_ONLY_FILES => no_empty_or_comments_only_files::check_with_files,
            VITEST_TEST_CORRESPONDENCE => vitest_test_correspondence::check_with_files,
            FILE_EXTENSION_POLICY => file_extension_policy::check_with_files,
            BANNED_RENAMED_FILES => banned_renamed_files::check_with_files,
            LOCKFILE_ALLOWLIST => lockfile_allowlist::check_with_files,
            DOC_CONSISTENCY => doc_consistency::check_with_files,
            SHELLCHECK_RUNNER => shellcheck_runner::check_with_files,
        }
    };
}

macro_rules! define_filesystem_rule_ids {
    ($($id:expr => $call:path),* $(,)?) => {
        const FILESYSTEM_RULE_IDS: &[&str] = &[
            $($id,)*
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_TESTS,
            RUST_NO_INLINE_ALLOWS,
        ];
    };
}

filesystem_rules!(define_filesystem_rule_ids);

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
    let preserved_roots =
        preserved::filesystem_rule_target_roots(root, &config, FILESYSTEM_RULE_IDS);
    let files = crate::codebase::ts_source::discover_files_preserving_roots(
        root,
        &config.filesystem.skip_directories,
        &preserved_roots,
    );
    run_filesystem_rules_with_config(root, &config, &files)
}

fn run_filesystem_rules_with_config(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let acc: RuleAcc = Mutex::new(Vec::new());
    macro_rules! run_rules {
        ($($id:expr => $call:path),* $(,)?) => {
            rayon::scope(|s| {
                $(
                    if rule_enabled(config, $id) {
                        s.spawn(|_| {
                            let rule_files = preserved::filesystem_rule_files(root, config, $id, files);
                            let res = $call(root, config, &rule_files);
                            acc.lock().unwrap_or_else(|e| e.into_inner()).push(($id, res));
                        });
                    }
                )*
                if rust_rules_enabled(config) {
                    s.spawn(|_| {
                        let res = rust_rules_combined::check_with_files(root, config, files);
                        acc.lock().unwrap_or_else(|e| e.into_inner()).push(("rust-rules-combined", res));
                    });
                }
            });
        };
    }
    filesystem_rules!(run_rules);
    let mut results = acc.into_inner().unwrap_or_else(|e| e.into_inner());
    results.sort_unstable_by_key(|(id, _)| *id);
    let mut findings = Vec::new();
    for (_, r) in results {
        findings.extend(r?);
    }
    suppress_rule_findings(root, &mut findings);
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn rust_rules_enabled(config: &crate::config::v2::NoMistakesConfig) -> bool {
    rule_enabled(config, RUST_MAX_LINES_PER_FILE)
        || rule_enabled(config, RUST_NO_INLINE_TESTS)
        || rule_enabled(config, RUST_NO_INLINE_ALLOWS)
}

#[cfg(test)]
mod tests;
