use super::prefix_globs;
use crate::config::v2::schema::NoMistakesConfig;
use std::collections::BTreeSet;
use std::path::Path;

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
    let paths = files
        .iter()
        .filter(|path| {
            let relative = crate::codebase::ts_source::relative_slash_path(root, path);
            filters.iter().any(|(include, exclude)| {
                include.is_match(&relative) && !exclude.is_match(&relative)
            })
        })
        .map(|path| {
            record_scope_normalization!();
            crate::codebase::ts_resolver::normalize_path(path)
        })
        .collect::<BTreeSet<_>>();
    if paths.len() == files.len() {
        FamilyPathScope::All
    } else {
        FamilyPathScope::Only(std::sync::Arc::new(paths))
    }
}
