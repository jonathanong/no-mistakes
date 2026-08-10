use crate::fetches::pipeline::cache::Cache;
use crate::fetches::pipeline::target::{route_matches_target, TargetSpec};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn check_route_matches(
    route: &no_mistakes::routes::Route,
    target_specs: &[TargetSpec],
    wrapper_files: &[PathBuf],
    mut context: RouteMatchContext<'_>,
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
            let reaches_route_target = reaches_target(&route.file, target_file, &mut context)?;
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

                let reaches_wrapper_target =
                    reaches_target(wrapper_file, target_file, &mut context)?;
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

pub(crate) struct RouteMatchContext<'a> {
    pub(crate) cache: &'a mut Cache,
    pub(crate) session: &'a no_mistakes::codebase::analysis_session::AnalysisSession,
    pub(crate) parsed_files: &'a mut no_mistakes::fetch::ParsedFileCache,
    pub(crate) root: &'a Path,
    pub(crate) visible_files: &'a HashSet<PathBuf>,
}

fn reaches_target(
    source_file: &Path,
    target_file: &Path,
    context: &mut RouteMatchContext<'_>,
) -> Result<bool> {
    let mut visited_targets = HashSet::new();
    let mut facts = no_mistakes::fetch::RouteTargetFacts {
        root: context.root,
        visited: &mut visited_targets,
        import_cache: &mut context.cache.imports,
        parsed_files: &mut *context.parsed_files,
        visible_files: context.visible_files,
    };
    no_mistakes::fetch::route_reaches_target_from_visible_with_facts_and_session(
        context.session,
        source_file,
        target_file,
        &mut facts,
    )
}
