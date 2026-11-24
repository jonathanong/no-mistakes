#[path = "sourced_gha.rs"]
mod gha;

use super::super::*;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Option<Result<Vec<RuleFinding>>> {
    if let Some(found) = crate::codebase::rules::github_actions_workflow_files::dispatch::run(
        rule_id, root, config, files, sources,
    ) {
        return Some(found);
    }
    if let Some(found) = gha::run(rule_id, root, config, files, sources, defer_suppression) {
        return Some(found);
    }
    Some(match rule_id {
        NO_MISTAKES_CONFIG => {
            no_mistakes_config::check_with_files_and_sources(root, config, files, sources)
        }
        INTEGRATION_TEST_NO_MOCKS => {
            integration_test_no_mocks::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_LINK_DISPLAY_TEXT => {
            markdown_link_display_text::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_EVAL_TESTS => {
            markdown_eval_tests::check_with_files_and_sources(root, config, files, sources)
        }
        DECLARED_PAYLOAD_COMPATIBILITY => {
            declared_payload_compatibility::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        STRUCTURED_CONFIG_POLICY => {
            structured_config_policy::check_with_files_and_sources(root, config, files, sources)
        }
        CONFIG_PATH_REFERENCES => {
            config_path_references::check_with_files_and_sources(root, config, files, sources)
        }
        TSCONFIG_FILE_COVERAGE => {
            tsconfig_file_coverage::check_with_files_and_sources(root, config, files, sources)
        }
        VITE_OPTIMIZE_DEPS_NESTED_IMPORTS => {
            vite_optimize_deps_nested_imports::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        TEST_FIXTURE_SIZE_CAP => test_fixture_size_cap::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        ),
        TEST_NO_RETRIES => test_no_retries::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        ),
        PLAYWRIGHT_NO_SERIAL => playwright_no_serial::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        ),
        NO_EMPTY_OR_COMMENTS_ONLY_FILES => {
            no_empty_or_comments_only_files::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        _ => return None,
    })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
