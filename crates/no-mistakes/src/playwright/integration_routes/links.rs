use super::RouteLink;
use crate::codebase::check_facts::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::ts_source::FileIdMap;
use crate::config::v2::schema::RouteCoverageSource;
use crate::integration_tests::{runner_config::ParsedRunnerConfigs, types::Framework};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

pub(crate) type PreparedLinks = BTreeMap<RouteCoverageSource, Result<Vec<RouteLink>, String>>;

pub(crate) fn prepare_links(
    root: &Path,
    plan: &CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    facts: &FileIdMap<CheckFileFacts>,
    configs: crate::codebase::check_facts::RunnerConfigFacts,
) -> PreparedLinks {
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized_root.as_path();
    let sources = playwright
        .integration_route_settings()
        .flat_map(|settings| settings.route_coverage_sources.iter().cloned())
        .collect::<BTreeSet<_>>();
    if sources.is_empty() {
        return BTreeMap::new();
    }
    let projects = plan
        .integration_runner_configs
        .as_ref()
        .ok_or_else(|| {
            anyhow::anyhow!("routeCoverageSources requires prepared Vitest runner configuration")
        })
        .and_then(|plan| {
            ParsedRunnerConfigs::with_files(configs).projects_for(plan, Framework::Vitest)
        });
    sources
        .into_iter()
        .map(|source| {
            let result = (|| {
                let projects = projects
                    .as_ref()
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                let matching_projects = projects
                    .iter()
                    .filter(|project| {
                        project.runner_project_arg.as_deref() == Some(&source.project)
                            || project.policy_name.as_deref() == Some(&source.project)
                    }).collect::<Vec<_>>();
                anyhow::ensure!(matching_projects.len() == 1,
                    "routeCoverageSources project `{}` must identify exactly one registered Vitest project", source.project);
                let project = matching_projects[0];
                let include = crate::playwright::fsutil::build_globset(&project.include)?;
                let exclude = crate::playwright::fsutil::build_globset(&project.exclude)?;
                let resolution = playwright.module_resolution().ok_or_else(|| {
                    anyhow::anyhow!("routeCoverageSources requires prepared module resolution")
                })?;
                let mut links = Vec::new();
                for (entry, _) in facts.iter().filter(|(path, _)| {
                    let relative = crate::playwright::fsutil::relative_string(root, path);
                    include.is_match(&relative) && !exclude.is_match(&relative)
                }) {
                    let mut seen = BTreeSet::new();
                    let mut pending = VecDeque::from([entry.clone()]);
                    while let Some(path) = pending.pop_front() {
                        if !seen.insert(path.clone()) {
                            continue;
                        }
                        let Some(file) = facts.get(&path) else {
                            continue;
                        };
                        anyhow::ensure!(
                            file.parse_error.is_none(),
                            "failed to parse registered integration module {}",
                            path.display()
                        );
                        for occurrence in file
                            .integration_route_occurrences
                            .iter()
                            .filter(|occurrence| occurrence.source == source)
                        {
                            links.push(RouteLink {
                                source: source.clone(),
                                entry_file: entry.clone(),
                                declaration_file: path.clone(),
                                occurrence: occurrence.occurrence.clone(),
                            });
                        }
                        for import in file
                            .ts
                            .imports
                            .iter()
                            .filter(|import| import.kind == ImportKind::Static)
                        {
                            if let Some(imported) =
                                resolution.resolved_path(&import.specifier, &path)
                            {
                                pending.push_back(imported);
                            }
                        }
                    }
                }
                links.sort();
                links.dedup();
                Ok(links)
            })()
            .map_err(|error: anyhow::Error| error.to_string());
            (source, result)
        })
        .collect()
}
