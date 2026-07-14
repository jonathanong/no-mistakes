use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

type RuleAcc = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;

use super::{
    agents_md_max_size, banned_paths, banned_renamed_files, config_path_references,
    doc_consistency, file_extension_policy, finite_set_consistency, forbidden_workspace_closure,
    github_actions_pinned_hash, integration_test_no_mocks, lockfile_allowlist,
    markdown_link_display_text, no_empty_or_comments_only_files, no_git_identity_mutation,
    package_json_registry_only, package_json_workspace_coverage, require_files_in_subdirs,
    require_test_per_subdir, required_companion_imports, required_local_docs, rust_rules_combined,
    shellcheck_runner, strict_package_layout, structured_config_policy, test_email_domain_policy,
    tsconfig_alias_folder_mapping, vitest_ci_path_coverage, vitest_project_mapping,
    vitest_test_correspondence, workspace_package_cycles,
};

mod candidate_index;
mod entrypoints;
mod preserved;
mod run_rule;
use super::{
    rule_enabled, suppress_rule_findings_with_sources_except, RuleFinding, AGENTS_MD_MAX_SIZE,
    BANNED_PATHS, BANNED_RENAMED_FILES, CONFIG_PATH_REFERENCES, DOC_CONSISTENCY,
    FILE_EXTENSION_POLICY, FINITE_SET_CONSISTENCY, FORBIDDEN_WORKSPACE_CLOSURE,
    INTEGRATION_TEST_NO_MOCKS, LOCKFILE_ALLOWLIST, MARKDOWN_LINK_DISPLAY_TEXT,
    NO_EMPTY_OR_COMMENTS_ONLY_FILES, NO_GIT_IDENTITY_MUTATION, PACKAGE_JSON_REGISTRY_ONLY,
    PACKAGE_JSON_WORKSPACE_COVERAGE, REQUIRED_COMPANION_IMPORTS, REQUIRED_DOC_SECTION,
    REQUIRED_LOCAL_DOCS, REQUIRE_FILES_IN_SUBDIRS, REQUIRE_TEST_PER_SUBDIR,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS, SHELLCHECK_RUNNER,
    STRICT_PACKAGE_LAYOUT, STRUCTURED_CONFIG_POLICY, TEST_EMAIL_DOMAIN_POLICY,
    TSCONFIG_ALIAS_FOLDER_MAPPING, VITEST_CI_PATH_COVERAGE, VITEST_PROJECT_MAPPING,
    VITEST_TEST_CORRESPONDENCE, WORKSPACE_PACKAGE_CYCLES,
};
pub use entrypoints::{
    run_filesystem_rules, run_filesystem_rules_with_config,
    run_filesystem_rules_with_config_and_snapshot,
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog, run_filesystem_rules_with_files,
};
const GITHUB_ACTIONS_PINNED_HASH: &str = github_actions_pinned_hash::RULE_ID;

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

macro_rules! define_filesystem_rule_ids {
    ($($id:expr => $call:path),* $(,)?) => {
        const FILESYSTEM_RULE_IDS: &[&str] = &[
            $($id,)*
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_TESTS,
            RUST_NO_INLINE_ALLOWS,
            VITEST_PROJECT_MAPPING,
            VITEST_CI_PATH_COVERAGE,
        ];
    };
}

filesystem_rules!(define_filesystem_rule_ids);

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_and_sources(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    vitest_catalog: Option<&super::PreparedVitestProjectCatalog>,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let acc: RuleAcc = Mutex::new(Vec::new());
    let metadata_files = if rule_enabled(config, FORBIDDEN_WORKSPACE_CLOSURE) {
        let mut metadata_files = files.to_vec();
        metadata_files.extend(snapshot.paths_for(root).iter().cloned());
        metadata_files.sort();
        metadata_files.dedup();
        metadata_files
    } else {
        Vec::new()
    };
    let candidates = candidate_index::RuleCandidateIndex::prepare_with_inventory(
        root,
        config,
        files,
        &metadata_files,
        Some(sources.inventory().paths()),
    );
    macro_rules! run_rules {
        ($($id:expr => $call:path),* $(,)?) => {
            rayon::scope(|s| {
                $(
                    if rule_enabled(config, $id) {
                        s.spawn(|_| {
                            let res = run_rule::run_rule_with_sources(
                                $id,
                                $call,
                                root,
                                config,
                                candidates.candidates($id),
                                &sources,
                            );
                            acc.lock().expect("mutex poisoned").push(($id, res));
                        });
                    }
                )*
                if rust_rules_enabled(config) {
                    s.spawn(|_| {
                        let res = rust_rules_combined::check_with_files_and_sources(
                            root,
                            config,
                            candidates.rust_candidates(),
                            candidates.exclusive_rust_candidates(),
                            &sources,
                        );
                        acc.lock().expect("mutex poisoned").push(("rust-rules-combined", res));
                    });
                }
                if rule_enabled(config, VITEST_PROJECT_MAPPING) {
                    s.spawn(|_| {
                        let res = vitest_project_mapping::check_with_files_and_catalog(
                            root,
                            config,
                            candidates.candidates(VITEST_PROJECT_MAPPING),
                            vitest_catalog,
                        );
                        acc.lock()
                            .expect("mutex poisoned")
                            .push((VITEST_PROJECT_MAPPING, res));
                    });
                }
                if rule_enabled(config, VITEST_CI_PATH_COVERAGE) {
                    s.spawn(|_| {
                        let res = vitest_ci_path_coverage::check_with_files_from_snapshot_catalog_and_sources(
                            root,
                            config,
                            candidates.candidates(VITEST_CI_PATH_COVERAGE),
                            snapshot,
                            vitest_catalog,
                            &sources,
                        );
                        acc.lock()
                            .expect("mutex poisoned")
                            .push((VITEST_CI_PATH_COVERAGE, res));
                    });
                }
            });
        };
    }
    filesystem_rules!(run_rules);
    let mut results = acc.into_inner().expect("mutex poisoned");
    results.sort_unstable_by_key(|(id, _)| *id);
    let mut findings = Vec::new();
    for (_, r) in results {
        findings.extend(r?);
    }
    suppress_rule_findings_with_sources_except(
        root,
        &mut findings,
        &sources,
        &[
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_TESTS,
            RUST_NO_INLINE_ALLOWS,
        ],
    );
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
