use crate::fetches::pipeline::cache::Cache;
use crate::fetches::pipeline::target::{route_matches_target, TargetSpec};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn check_route_matches(
    route: &no_mistakes::routes::Route,
    target_specs: &[TargetSpec],
    wrapper_files: &[PathBuf],
    cache: &mut Cache,
    parsed_files: &mut no_mistakes::fetch::ParsedFileCache,
    root: &Path,
    visible_files: &HashSet<PathBuf>,
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
            let reaches_route_target = reaches_target(
                &route.file,
                target_file,
                root,
                cache,
                parsed_files,
                visible_files,
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

                let reaches_wrapper_target = reaches_target(
                    wrapper_file,
                    target_file,
                    root,
                    cache,
                    parsed_files,
                    visible_files,
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

fn reaches_target(
    source_file: &Path,
    target_file: &Path,
    root: &Path,
    cache: &mut Cache,
    parsed_files: &mut no_mistakes::fetch::ParsedFileCache,
    visible_files: &HashSet<PathBuf>,
) -> Result<bool> {
    let mut visited_targets = HashSet::new();
    no_mistakes::fetch::route_reaches_target_from_visible_with_facts(
        source_file,
        target_file,
        root,
        &mut visited_targets,
        &mut cache.imports,
        parsed_files,
        visible_files,
    )
}
