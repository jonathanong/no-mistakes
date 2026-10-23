use super::FamilyPathScope;
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::rules::NEXTJS_NO_CACHING;
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn nextjs_scope(
    root: &Path,
    files: &[PathBuf],
    config: &NoMistakesConfig,
    inferred: &mut crate::codebase::config::InferredRoots,
) -> Result<FamilyPathScope> {
    let root = normalize_path(root);
    let mut applications = Vec::new();
    let mut any = false;
    let mut unrestricted = false;
    for rule in config.rule_applications(NEXTJS_NO_CACHING) {
        any = true;
        let roots =
            crate::codebase::rules::target_roots_with_inferred(&root, config, rule, inferred);
        let filter = crate::codebase::rules::path_filter::RulePathFilter::new_with_inferred(
            &root, config, rule, inferred,
        )?;
        // Validate every application's filter even when broad demand wins.
        unrestricted |=
            rule.applies_to_repository() && rule.include.is_empty() && rule.exclude.is_empty();
        if unrestricted {
            continue;
        }
        applications.push((roots, filter));
    }
    if !any || unrestricted {
        return Ok(FamilyPathScope::All);
    }
    if files.is_empty() {
        return Ok(FamilyPathScope::Only(Arc::new(BTreeSet::new())));
    }
    // Graph consumers may add skipped-directory files. Match the complete
    // supplied inventory, rather than the primary selected_rule_files set.
    let matches = |path: &PathBuf| {
        applications.iter().any(|(roots, filter)| {
            roots.iter().any(|root| path.starts_with(root)) && filter.is_match(path)
        })
    };
    let Some(first_excluded) = files.iter().position(|path| !matches(path)) else {
        // A fully selected inventory needs no owned membership paths.
        return Ok(FamilyPathScope::All);
    };
    // The prefix was already matched. Materialize it only after proving that
    // this is a partial scope, and evaluate each remaining path exactly once.
    let selected = files[..first_excluded]
        .iter()
        .chain(
            files[first_excluded + 1..]
                .iter()
                .filter(|path| matches(path)),
        )
        .map(|path| normalized_membership_path(path))
        .collect();
    Ok(FamilyPathScope::Only(Arc::new(selected)))
}

fn normalized_membership_path(path: &Path) -> PathBuf {
    let normalized = normalize_path(path);
    #[cfg(test)]
    MEMBERSHIP_NORMALIZATIONS.with(|count| count.set(count.get() + 1));
    normalized
}

#[cfg(test)]
thread_local! {
    static MEMBERSHIP_NORMALIZATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn take_membership_normalizations() -> usize {
    MEMBERSHIP_NORMALIZATIONS.with(|count| count.replace(0))
}

/// Scope file-local caching facts over the complete visible inventory, including
/// graph-only files, while borrowing the request's prepared framework inference.
pub fn configure_nextjs_caching_scope(
    root: &Path,
    visible: &[PathBuf],
    config: &NoMistakesConfig,
    inferred: &crate::codebase::config::InferredRoots,
    plan: &mut CheckFactPlan,
) -> Result<()> {
    if plan.nextjs_caching || plan.nextjs_caching_low {
        plan.nextjs_caching_paths = nextjs_scope(root, visible, config, &mut inferred.clone())?;
    }
    Ok(())
}
