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
    Some(match rule_id {
        STRUCTURED_CONFIG_POLICY => structured_config_policy::diagnostics_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        )
        .map(|output| output.findings),
        NO_EMPTY_OR_COMMENTS_ONLY_FILES => {
            no_empty_or_comments_only_files::diagnostics_with_files_and_sources(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
            .map(|output| output.findings)
        }
        GITHUB_ACTIONS_PINNED_HASH => {
            github_actions_pinned_hash::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        CSHARP_MAX_LINES_PER_FILE => {
            csharp_max_lines_per_file::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        CSHARP_NO_ASYNC_VOID_DELEGATE => {
            csharp_no_async_void_delegate::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        SWIFT_NO_RAW_PRINT => {
            swift_no_raw_print::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        SWIFT_VIEWMODEL_MAIN_ACTOR => {
            swift_viewmodel_main_actor::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        VERSION_PIN_CONSISTENCY => {
            version_pin_consistency::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        NO_RAW_EPHEMERAL_PORT => {
            no_raw_ephemeral_port::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        _ => return None,
    })
}
