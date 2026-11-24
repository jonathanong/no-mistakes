use super::{framework_plan, relative_path, test_runner, PreparedTestPlanRequest, TestFramework};
use globset::{Glob, GlobSetBuilder};
use no_mistakes::changes::config::ChangeCategory;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::ts_source::FixtureReadIndex;
use rayon::prelude::*;
use regex::RegexSet;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) struct CategoryMatches<'a> {
    pub(super) tests: Vec<(&'a Path, Vec<&'a str>)>,
    pub(super) unmatched: Vec<&'a str>,
}

impl<'a> CategoryMatches<'a> {
    pub(super) fn find(
        prepared: &PreparedTestPlanRequest,
        discovered: &'a DiscoveredTests,
        name: &str,
        category: &ChangeCategory,
        ids: &BTreeSet<&'a str>,
        attributes: Option<(&FixtureReadIndex, &BTreeMap<String, Vec<String>>)>,
    ) -> Result<Self, String> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &category.tests {
            builder.add(
                Glob::new(pattern.trim_start_matches("./")).map_err(|error| {
                    format!("Invalid test glob in changes category `{name}`: {error}")
                })?,
            );
        }
        let tests = builder
            .build()
            .map_err(|error| format!("Invalid test globs in changes category `{name}`: {error}"))?;
        let template = category
            .match_pattern
            .as_deref()
            .unwrap_or("{id}")
            .replace("<id>", "{id}");
        if !template.contains("{id}") || ids.contains("") {
            return Err(format!(
                "Invalid ID or match pattern in changes category `{name}`"
            ));
        }
        let ids = ids.iter().copied().collect::<Vec<_>>();
        let literals: Vec<_> = ids.iter().map(|id| template.replace("{id}", id)).collect();
        let matcher = RegexSet::new(literals.iter().map(|literal| regex::escape(literal)))
            .map_err(|error| {
                format!("Cannot build ID matcher for changes category `{name}`: {error}")
            })?;
        let attributes =
            attributes.filter(|_| super::attributes::loader_template(category).is_some());
        let allow_parse = !prepared.graph_is_initialized();
        let found: Vec<_> = discovered
            .tests
            .par_iter()
            .map(|path| (path, relative_path(&prepared.root, path)))
            .filter(|(_, rel)| tests.is_match(rel))
            .map(|(path, rel)| {
                prepared.source_store().read_path(path).map_or_else(
                    |_| {
                        Err(format!(
                            "Cannot read test `{rel}` for changes category `{name}`"
                        ))
                    },
                    |source| {
                        let indexes = matcher
                            .matches(&source)
                            .into_iter()
                            .map(|found| {
                                let selected = attributes.is_none_or(|(reads, paths)| {
                                    paths.get(ids[found]).is_none_or(|changed| {
                                        reads.affects(
                                            path,
                                            &source,
                                            &literals[found],
                                            changed,
                                            allow_parse,
                                        )
                                    })
                                });
                                (found, selected)
                            })
                            .collect::<Vec<_>>();
                        Ok((path.as_path(), indexes))
                    },
                )
            })
            .collect();
        let mut matched = vec![false; ids.len()];
        let mut tests = Vec::new();
        for entry in found {
            let (path, indexes) = entry?;
            let mut selected = Vec::new();
            for (index, affects_test) in indexes {
                matched[index] = true;
                if affects_test {
                    selected.push(ids[index]);
                }
            }
            if !selected.is_empty() {
                tests.push((path, selected));
            }
        }
        let unmatched = ids
            .into_iter()
            .zip(matched)
            .filter_map(|(id, found)| (!found).then_some(id))
            .collect();
        Ok(Self { tests, unmatched })
    }
}

pub(super) type RunnerTargets = BTreeMap<&'static str, BTreeSet<String>>;

pub(super) fn fallback_targets(
    prepared: &PreparedTestPlanRequest,
    frameworks: &[TestFramework],
    category: &ChangeCategory,
    discovered: &DiscoveredTests,
) -> Result<Option<RunnerTargets>, String> {
    let Some(name) = category.fallback.as_deref() else {
        return Ok(None);
    };
    if name == "full-suite" {
        return Ok(None);
    }
    let mut targets = RunnerTargets::new();
    for &framework in frameworks {
        let Some(trigger) = framework_plan(&prepared.config, framework)
            .full_suite_triggers
            .triggers
            .iter()
            .find(|trigger| trigger.name == name)
        else {
            continue;
        };
        if trigger.targets.is_empty() {
            return Ok(None);
        }
        targets.insert(
            test_runner(framework).as_str(),
            trigger.targets.iter().cloned().collect(),
        );
    }
    if targets.is_empty() {
        return Err(format!("No fallback trigger named `{name}`"));
    }
    let discovered_projects = discovered
        .targets_by_path
        .values()
        .flatten()
        .filter_map(|target| Some((target.runner.as_str(), target.project.as_deref()?)))
        .collect::<BTreeSet<_>>();
    for (runner, projects) in &targets {
        for project in projects {
            if !discovered_projects.contains(&(*runner, project.as_str())) {
                return Err(format!(
                    "Fallback trigger `{name}` has no discovered tests for {runner} project `{project}`"
                ));
            }
        }
    }
    Ok(Some(targets))
}
