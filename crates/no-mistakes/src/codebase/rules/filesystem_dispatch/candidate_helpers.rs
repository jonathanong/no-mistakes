use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::codebase::rules::{
    BANNED_PATHS, BANNED_RENAMED_FILES, CONFIG_PATH_REFERENCES, DOC_CONSISTENCY,
    FILE_EXTENSION_POLICY, FINITE_SET_CONSISTENCY, INTEGRATION_TEST_NO_MOCKS,
    NO_EMPTY_OR_COMMENTS_ONLY_FILES, NO_GIT_IDENTITY_MUTATION, REQUIRED_COMPANION_IMPORTS,
    SHELLCHECK_RUNNER, STRUCTURED_CONFIG_POLICY, TEST_EMAIL_DOMAIN_POLICY,
};

pub(super) fn is_rust_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("rs")
}

// Rules that may read Rust directly or emit a Rust-path finding whose
// suppression check must read the source from the shared store.
pub(super) fn rule_can_consume_rust_source(rule_id: &str) -> bool {
    matches!(
        rule_id,
        BANNED_PATHS
            | BANNED_RENAMED_FILES
            | CONFIG_PATH_REFERENCES
            | DOC_CONSISTENCY
            | FILE_EXTENSION_POLICY
            | FINITE_SET_CONSISTENCY
            | INTEGRATION_TEST_NO_MOCKS
            | NO_EMPTY_OR_COMMENTS_ONLY_FILES
            | NO_GIT_IDENTITY_MUTATION
            | REQUIRED_COMPANION_IMPORTS
            | SHELLCHECK_RUNNER
            | STRUCTURED_CONFIG_POLICY
            | TEST_EMAIL_DOMAIN_POLICY
    )
}

pub(super) fn normalized_paths(paths: &[PathBuf]) -> Cow<'_, [PathBuf]> {
    let already_normalized = paths.windows(2).all(|pair| pair[0] < pair[1])
        && paths.iter().all(|path| {
            !path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::CurDir | std::path::Component::ParentDir
                )
            })
        });
    if already_normalized {
        return Cow::Borrowed(paths);
    }
    let mut normalized = paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    Cow::Owned(normalized)
}

pub(super) fn markdown_inventory_path_allowed(
    request_root: &Path,
    path: &Path,
    roots: &[PathBuf],
    skip: &HashSet<&str>,
) -> bool {
    // Baselines are JSON companions rather than documentation targets, so
    // retain them for the rule's tracked-baseline validation.
    if path
        .extension()
        .is_none_or(|extension| !matches!(extension.to_str(), Some("md" | "markdown" | "mdx")))
    {
        return true;
    }
    !crate::codebase::ts_source::is_under_skipped_dir(request_root, path, skip)
        && roots.iter().any(|root| {
            path.starts_with(root)
                && !crate::codebase::ts_source::is_under_skipped_dir(root, path, skip)
        })
}
