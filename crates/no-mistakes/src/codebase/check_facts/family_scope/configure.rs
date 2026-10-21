use super::closure::with_import_closure;
use super::rules::from_rule_ids;
use super::FamilyPathScope;
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::rules::{REQUIRE_STORYBOOK_STORIES, SERVER_ROUTE_CLIENT_BOUNDARY};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use crate::codebase::unique_exports::RULE_ID as UNIQUE_EXPORTS;
use crate::config::v2::NoMistakesConfig;
use crate::queue::extract_prefilter::source_may_have_queue_facts;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn configure_family_path_scopes(
    root: &Path,
    files: &[PathBuf],
    config: &NoMistakesConfig,
    sources: &SourceStore,
    plan: &mut CheckFactPlan,
) -> Result<()> {
    if plan.symbols {
        plan.symbols_paths = symbols_scope(root, files, sources, config, plan.storybook)?;
    }
    if plan.storybook {
        plan.storybook_paths = scoped_or_closure(
            root,
            files,
            sources,
            from_rule_ids(root, files, config, &[REQUIRE_STORYBOOK_STORIES])?,
        );
    }
    if plan.server_route_client_boundary {
        plan.boundary_paths = from_rule_ids(root, files, config, &[SERVER_ROUTE_CLIENT_BOUNDARY])?;
    }
    if plan.react {
        // react_traits default targets expand from the repository root; frontend_root
        // is only a fallback when that expansion is empty.
        plan.react_paths = FamilyPathScope::All;
    }
    if plan.queue {
        plan.queue_paths = queue_scope(root, files, sources, plan);
    }
    if plan.integration {
        plan.integration_paths = FamilyPathScope::All;
    }
    Ok(())
}

fn symbols_scope(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    config: &NoMistakesConfig,
    storybook: bool,
) -> Result<FamilyPathScope> {
    // unique-exports follows reexport origins that can sit outside projects.
    let unique = scoped_or_closure(
        root,
        files,
        sources,
        from_rule_ids(root, files, config, &[UNIQUE_EXPORTS])?,
    );
    if !storybook {
        return Ok(unique);
    }
    Ok(super::merge(
        true,
        &unique,
        true,
        &scoped_or_closure(
            root,
            files,
            sources,
            from_rule_ids(root, files, config, &[REQUIRE_STORYBOOK_STORIES])?,
        ),
    ))
}

fn scoped_or_closure(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    seeds: FamilyPathScope,
) -> FamilyPathScope {
    match seeds {
        FamilyPathScope::All => FamilyPathScope::All,
        FamilyPathScope::Only(paths) => {
            with_import_closure(root, files, sources, paths.as_ref().clone())
        }
    }
}

fn queue_scope(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    plan: &CheckFactPlan,
) -> FamilyPathScope {
    let mut seeds = BTreeSet::new();
    for path in files {
        let Ok(source) = sources.read_path(path) else {
            continue;
        };
        if source_may_have_queue_facts(
            &source,
            &plan.queue_factory_names,
            &plan.queue_factory_roles,
        ) {
            seeds.insert(normalize_path(path));
        }
    }
    with_import_closure(root, files, sources, seeds)
}
