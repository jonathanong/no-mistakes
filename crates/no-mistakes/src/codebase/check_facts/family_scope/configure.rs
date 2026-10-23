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
use std::sync::Arc;

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
        plan.storybook_paths = storybook_scope(root, files, sources, config)?;
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
        plan.integration_test_paths =
            crate::integration_tests::config::test_discovery_scope(root, files, config);
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
        &storybook_scope(root, files, sources, config)?,
    ))
}

fn storybook_scope(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    config: &NoMistakesConfig,
) -> Result<FamilyPathScope> {
    let selected = from_rule_ids(root, files, config, &[REQUIRE_STORYBOOK_STORIES])?;
    // Component include/project filters do not reach stories that import those
    // components. Coverage reads story facts independently via story patterns.
    let mut seeds = crate::codebase::rules::require_storybook_stories::configured_story_files(
        root, files, config, sources,
    )?;
    match selected {
        FamilyPathScope::All => Ok(FamilyPathScope::All),
        FamilyPathScope::Only(paths) => {
            seeds.extend(Arc::unwrap_or_clone(paths));
            Ok(with_import_closure(root, files, sources, seeds))
        }
    }
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
            with_import_closure(root, files, sources, Arc::unwrap_or_clone(paths))
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
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
        ) {
            continue;
        }
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
