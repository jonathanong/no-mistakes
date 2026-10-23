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
    let mut selected = BTreeSet::new();
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
        // Graph consumers may add files in filesystem.skipDirectories. The rule
        // historically checks those files too, so do not use selected_rule_files.
        selected.extend(
            files
                .iter()
                .filter(|path| {
                    roots.iter().any(|root| path.starts_with(root)) && filter.is_match(path)
                })
                .map(|path| normalize_path(path)),
        );
    }
    if !any || unrestricted {
        return Ok(FamilyPathScope::All);
    }
    // Unlike cross-file closure scopes, an empty file-local selection is exact.
    Ok(if selected.is_empty() {
        FamilyPathScope::Only(Arc::new(selected))
    } else {
        super::closure::only_or_all(selected, files.len())
    })
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
