use crate::analyze::file::analyze_file;
use crate::analyze::routes::{is_route_handler_file, route_reaches_target};
use crate::pipeline::cache::Cache;
use crate::pipeline::target::{route_matches_target, TargetSpec};
use crate::report::types::FetchOccurrence;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn check_route_matches(
    route: &no_mistakes_core::routes::Route,
    target_specs: &[TargetSpec],
    wrapper_files: &[PathBuf],
    cache: &mut Cache,
) -> Result<(bool, Vec<String>)> {
    let mut newly_matched = Vec::new();

    if target_specs.is_empty() {
        return Ok((true, newly_matched));
    }

    let mut matched = false;
    'target_match: for target in target_specs {
        if route_matches_target(&route.pattern, &target.raw) {
            matched = true;
            newly_matched.push(target.raw.clone());
            continue;
        }

        if let Some(target_file) = &target.file {
            let mut visited_targets = HashSet::new();
            let reaches_route_target = route_reaches_target(
                &route.file,
                target_file,
                &mut visited_targets,
                &mut cache.imports,
            )?;
            if reaches_route_target {
                matched = true;
                newly_matched.push(target.raw.clone());
                continue 'target_match;
            }

            let mut wrapper_file_matches = false;
            for wrapper_file in wrapper_files {
                if wrapper_file == target_file {
                    wrapper_file_matches = true;
                    break;
                }

                let mut wrapper_targets = HashSet::new();
                let reaches_wrapper_target = route_reaches_target(
                    wrapper_file,
                    target_file,
                    &mut wrapper_targets,
                    &mut cache.imports,
                )?;
                if reaches_wrapper_target {
                    wrapper_file_matches = true;
                    break;
                }
            }

            if wrapper_file_matches {
                matched = true;
                newly_matched.push(target.raw.clone());
                continue 'target_match;
            }
        }
    }

    Ok((matched, newly_matched))
}

pub(crate) fn collect_route_fetches(
    route: &no_mistakes_core::routes::Route,
    frontend_root: &Path,
    root: &Path,
    cache: &mut Cache,
) -> Result<Vec<FetchOccurrence>> {
    let route_is_page = route.file.file_stem().and_then(|s| s.to_str()) == Some("page");
    let route_is_route_handler = is_route_handler_file(&route.file);

    let mut visited = HashSet::new();
    let mut fetches = Vec::new();

    let _route_is_client = analyze_file(
        &route.file,
        root,
        &mut visited,
        &mut fetches,
        cache,
        false,
        route_is_route_handler,
    )?;

    if route_is_page {
        collect_page_layout_fetches(
            route,
            frontend_root,
            root,
            cache,
            &mut visited,
            &mut fetches,
        )?;
    }

    fetches.sort();
    Ok(fetches)
}

fn collect_page_layout_fetches(
    route: &no_mistakes_core::routes::Route,
    frontend_root: &Path,
    root: &Path,
    cache: &mut Cache,
    visited: &mut HashSet<(PathBuf, bool, bool)>,
    fetches: &mut Vec<FetchOccurrence>,
) -> Result<()> {
    let route_is_route_handler = is_route_handler_file(&route.file);
    let mut current = route.file.parent();
    while let Some(parent) = current {
        if !parent.starts_with(frontend_root) {
            break;
        }

        for stem in ["layout", "loading", "error", "not-found", "template"] {
            for ext in ["tsx", "ts", "jsx", "js"] {
                let layout_file = parent.join(format!("{stem}.{ext}"));
                if layout_file.exists() {
                    analyze_file(
                        &layout_file,
                        root,
                        visited,
                        fetches,
                        cache,
                        false,
                        route_is_route_handler,
                    )?;
                }
            }
        }
        current = parent.parent();
    }
    Ok(())
}
