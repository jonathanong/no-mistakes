use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn is_rust_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("rs")
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

pub(super) struct CandidateClass {
    pub(super) includes_metadata: bool,
    pub(super) tracked_only: bool,
    pub(super) includes_repository_inventory: bool,
    pub(super) inventory_only: bool,
}

pub(super) fn classify(rule_id: &str, repository_scoped: bool) -> CandidateClass {
    use super::{
        AST_GREP_IGNORE_FRESHNESS, BANNED_PATHS, CONFIG_PATH_REFERENCES,
        FORBIDDEN_WORKSPACE_CLOSURE, MARKDOWN_MERMAID_VALIDATION, MARKDOWN_REACHABILITY,
        MARKDOWN_STRUCTURE_BUDGET, NO_MISTAKES_CONFIG, PACKAGE_JSON_NESTED_WORKSPACE_COVERAGE,
        PRODUCTION_DEPENDENCY_DECLARATIONS, TSCONFIG_FILE_COVERAGE, VERSION_PIN_CONSISTENCY,
    };
    let markdown = matches!(
        rule_id,
        MARKDOWN_MERMAID_VALIDATION | MARKDOWN_REACHABILITY | MARKDOWN_STRUCTURE_BUDGET
    );
    CandidateClass {
        includes_metadata: rule_id == FORBIDDEN_WORKSPACE_CLOSURE
            || rule_id == PRODUCTION_DEPENDENCY_DECLARATIONS,
        tracked_only: rule_id == BANNED_PATHS
            || rule_id == TSCONFIG_FILE_COVERAGE
            || rule_id == VERSION_PIN_CONSISTENCY
            || matches!(
                rule_id,
                CONFIG_PATH_REFERENCES
                    | AST_GREP_IGNORE_FRESHNESS
                    | NO_MISTAKES_CONFIG
                    | PACKAGE_JSON_NESTED_WORKSPACE_COVERAGE
            ),
        includes_repository_inventory: (rule_id == BANNED_PATHS && repository_scoped)
            || markdown
            || matches!(
                rule_id,
                CONFIG_PATH_REFERENCES | AST_GREP_IGNORE_FRESHNESS | TSCONFIG_FILE_COVERAGE
            ),
        inventory_only: markdown,
    }
}

pub(super) fn markdown_inventory_path_allowed(
    request_root: &Path,
    path: &Path,
    roots: &[PathBuf],
    skip: &HashSet<&str>,
) -> bool {
    // Baselines are JSON companions rather than documentation targets, so
    // retain them for the rule's tracked-baseline validation.
    if !super::super::markdown_scope::is_mermaid_document(path) {
        return true;
    }
    !crate::codebase::ts_source::is_under_skipped_dir(request_root, path, skip)
        && roots.iter().any(|root| {
            path.starts_with(root)
                && !crate::codebase::ts_source::is_under_skipped_dir(root, path, skip)
        })
}
