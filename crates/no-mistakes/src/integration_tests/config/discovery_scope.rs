use super::prefix_globs;
use crate::config::v2::schema::NoMistakesConfig;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

#[cfg(not(test))]
macro_rules! record_scope_normalization {
    () => {};
}
#[cfg(test)]
#[macro_use]
mod tests;

/// Explicit policy includes are already repository-relative effective suite
/// globs. Runner-derived projects stay broad until their parsed configs exist;
/// invalid globs also retain the authoritative check's validation order.
pub(crate) fn test_discovery_scope(
    root: &Path,
    files: &[std::path::PathBuf],
    config: &NoMistakesConfig,
) -> crate::codebase::check_facts::FamilyPathScope {
    use crate::codebase::check_facts::FamilyPathScope;
    let mut filters = Vec::new();
    for policy in config
        .tests
        .vitest
        .projects
        .values()
        .chain(config.tests.playwright.projects.values())
    {
        if policy.integration_suites.is_empty() {
            continue;
        }
        if policy.include.is_empty() {
            return FamilyPathScope::All;
        }
        let (Ok(include), Ok(exclude)) = (
            super::super::project_config::build_globset(&prefix_globs(root, root, &policy.include)),
            super::super::project_config::build_globset(&prefix_globs(root, root, &policy.exclude)),
        ) else {
            return FamilyPathScope::All;
        };
        filters.push((include, exclude));
    }
    if filters.is_empty() {
        return FamilyPathScope::All;
    }
    let selected = |path: &PathBuf| {
        let relative = crate::codebase::ts_source::relative_slash_path(root, path);
        filters
            .iter()
            .any(|(include, exclude)| include.is_match(&relative) && !exclude.is_match(&relative))
    };
    let mut canonical_unique = true;
    let first_excluded = files.iter().enumerate().position(|(index, path)| {
        if canonical_unique {
            canonical_unique = (index == 0 || files[index - 1].as_path() < path.as_path())
                && path.components().all(|component| {
                    !matches!(
                        component,
                        Component::CurDir | Component::ParentDir | Component::Prefix(_)
                    )
                });
        }
        !selected(path)
    });
    if let Some(index) = first_excluded {
        let paths = files[..index]
            .iter()
            .chain(files[index + 1..].iter().filter(|path| selected(path)))
            .map(|path| normalize_selected(path))
            .collect();
        return FamilyPathScope::Only(std::sync::Arc::new(paths));
    }
    if canonical_unique {
        return FamilyPathScope::All;
    }
    // Arbitrary direct slices may contain duplicates or lexical aliases. Keep
    // the previous normalized cardinality, including platform prefix behavior.
    let paths: BTreeSet<_> = files.iter().map(|path| normalize_selected(path)).collect();
    if paths.len() == files.len() {
        FamilyPathScope::All
    } else {
        FamilyPathScope::Only(std::sync::Arc::new(paths))
    }
}

fn normalize_selected(path: &Path) -> PathBuf {
    record_scope_normalization!();
    crate::codebase::ts_resolver::normalize_path(path)
}
