//! Conservative impact handling for Vitest setup declarations that cannot be
//! resolved statically. These declarations deliberately stay out of the
//! ordinary dependency graph, but must not produce a confidently empty plan.

use crate::integration_tests::types::{ConfigProject, VitestSetupDependency};
use crate::tests::configured_plan_candidates::{merge_selected, selected_from_paths, stable_take};
use crate::tests::plan::relative_path;
use crate::tests::{SelectedTest, TestFramework, TestPlanGroupResult, Warning};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(crate) fn warnings(root: &Path, projects: Option<&[ConfigProject]>) -> Vec<Warning> {
    let mut warnings = projects
        .unwrap_or_default()
        .iter()
        .flat_map(|project| {
            project
                .vitest_setup
                .iter()
                .filter(|setup| is_unsafe(setup))
                .map(|setup| warning(root, project, setup))
        })
        .collect::<Vec<_>>();
    warnings.sort_by(|left, right| {
        (&left.file, &left.r#type, &left.message).cmp(&(&right.file, &right.r#type, &right.message))
    });
    warnings.dedup_by(|left, right| {
        left.r#type == right.r#type && left.file == right.file && left.message == right.message
    });
    warnings
}

pub(super) fn framework_warnings(
    framework: TestFramework,
    root: &Path,
    projects: Option<&[ConfigProject]>,
) -> Vec<Warning> {
    if framework == TestFramework::Vitest {
        warnings(root, projects)
    } else {
        Vec::new()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_selection(
    framework: TestFramework,
    root: &Path,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    projects: Option<&[ConfigProject]>,
    discovered: &DiscoveredTests,
    used: &mut HashSet<String>,
    selected_map: &mut BTreeMap<PathBuf, SelectedTest>,
    group_results: &mut Vec<TestPlanGroupResult>,
    fallback_reasons: &mut Vec<String>,
    global_limit: usize,
    has_global_limit: bool,
    all_test_count: usize,
) {
    if framework != TestFramework::Vitest || all_test_count == 0 {
        return;
    }
    let fallback_remaining = global_limit.saturating_sub(used.len());
    let Some((reason, picked)) = selection(
        root,
        changed_files,
        deleted_files,
        projects,
        discovered,
        used,
        fallback_remaining,
    ) else {
        return;
    };
    for test in &picked {
        used.insert(test.test_file.clone());
        selected_map
            .entry(root.join(&test.test_file))
            .and_modify(|entry| merge_selected(entry, test))
            .or_insert_with(|| test.clone());
    }
    if !picked.is_empty() {
        group_results.push(TestPlanGroupResult {
            r#type: "dependencies".to_string(),
            selected: picked.iter().map(|test| test.test_file.clone()).collect(),
            remaining: all_test_count.saturating_sub(used.len()),
            limit: has_global_limit.then_some(fallback_remaining),
        });
    }
    fallback_reasons.push(reason);
}

/// Select a conservative, project-bounded fallback only after an unsafe setup
/// declaration is relevant to this change. `used` is applied before the global
/// plan limit so ordinary candidates retain their established priority.
pub(crate) fn selection(
    root: &Path,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    projects: Option<&[ConfigProject]>,
    discovered: &DiscoveredTests,
    used: &HashSet<String>,
    limit: usize,
) -> Option<(String, Vec<SelectedTest>)> {
    let projects = projects?;
    let mut candidates = BTreeMap::new();
    let mut selected_owner = false;
    let mut selected_framework = false;
    let mut triggered = false;
    let mut unresolved_triggered = false;
    let mut deleted_resolved_triggered = false;
    let mut resolved_config_triggered = false;

    for project in projects {
        for setup in &project.vitest_setup {
            let unsafe_setup = is_unsafe(setup);
            let (triggers, resolved_config, deleted_resolved) = if unsafe_setup {
                (
                    trigger_paths(root, project, setup, changed_files, deleted_files),
                    false,
                    false,
                )
            } else {
                let config = resolved_config_trigger_paths(setup, changed_files, deleted_files);
                let deleted = deleted_transitive_trigger_paths(setup, deleted_files);
                let config_triggered = !config.is_empty();
                let deleted_triggered = !deleted.is_empty();
                (
                    config.into_iter().chain(deleted).collect(),
                    config_triggered,
                    deleted_triggered,
                )
            };
            if triggers.is_empty() {
                continue;
            }
            triggered = true;
            if unsafe_setup {
                unresolved_triggered = true;
            } else {
                deleted_resolved_triggered |= deleted_resolved;
            }
            resolved_config_triggered |= resolved_config;
            let owner_tests = owner_tests(root, project, discovered);
            // An owner is established by the parsed project identity, not by
            // whether that owner happens to have an eligible test in this
            // environment. Reclassifying an empty, known owner as framework
            // wide would let a disabled/excluded project select unrelated
            // tests.
            let owner_known = owner_is_known(project);
            let fallback_tests = if owner_known {
                &owner_tests
            } else {
                &discovered.tests
            };
            selected_owner |= owner_known;
            selected_framework |= !owner_known;
            for trigger in triggers {
                for candidate in selected_from_paths(
                    root,
                    fallback_tests,
                    "Vitest setup fallback",
                    Some(&trigger),
                ) {
                    if used.contains(&candidate.test_file) {
                        continue;
                    }
                    candidates
                        .entry(candidate.test_file.clone())
                        .and_modify(|existing| merge_selected(existing, &candidate))
                        .or_insert(candidate);
                }
            }
        }
    }

    if !triggered {
        return None;
    }
    let scope = match (selected_owner, selected_framework) {
        (true, false) => "selected owning project tests",
        (false, true) => "selected discovered Vitest tests",
        (true, true) => "selected owning project and discovered Vitest tests",
        (false, false) => unreachable!("a relevant Vitest setup fallback has an owner decision"),
    };
    let reason = match (
        unresolved_triggered,
        deleted_resolved_triggered,
        resolved_config_triggered,
    ) {
        (false, false, true) => format!("Vitest setup configuration changed; {scope}"),
        (true, false, false) => {
            format!("Vitest setup dependencies could not be resolved statically; {scope}")
        }
        (false, true, false) => {
            format!("A transitive dependency of a resolved setup was deleted; {scope}")
        }
        (true, true, false) => format!(
            "Vitest setup dependencies could not be resolved statically and a transitive dependency of a resolved setup was deleted; {scope}"
        ),
        (true, false, true) => format!(
            "Vitest setup dependencies could not be resolved statically and setup configuration changed; {scope}"
        ),
        (false, true, true) => format!(
            "A transitive dependency of a resolved setup was deleted and setup configuration changed; {scope}"
        ),
        (true, true, true) => format!(
            "Vitest setup dependencies could not be resolved statically, a transitive dependency of a resolved setup was deleted, and setup configuration changed; {scope}"
        ),
        (false, false, false) => unreachable!("a relevant Vitest setup fallback has a trigger type"),
    };
    Some((
        reason,
        stable_take(candidates.into_values().collect(), limit),
    ))
}

fn resolved_config_trigger_paths(
    setup: &VitestSetupDependency,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
) -> Vec<PathBuf> {
    changed_files
        .iter()
        .chain(deleted_files)
        .map(|path| normalize(path))
        .filter(|changed| {
            setup.trigger_paths.iter().any(|trigger| {
                changed == &normalize(trigger)
                    && !setup.resolver_candidate_paths.contains(trigger)
                    && setup
                        .resolved_path
                        .as_ref()
                        .is_none_or(|resolved| normalize(resolved) != *changed)
            })
        })
        .collect()
}

fn deleted_transitive_trigger_paths(
    setup: &VitestSetupDependency,
    deleted_files: &[PathBuf],
) -> Vec<PathBuf> {
    deleted_files
        .iter()
        .map(|path| normalize(path))
        .filter(|deleted| {
            setup
                .transitive_trigger_paths
                .iter()
                .any(|trigger| deleted == &normalize(trigger))
        })
        .collect()
}

fn is_unsafe(setup: &VitestSetupDependency) -> bool {
    setup.specifier.is_none() || setup.resolved_path.is_none()
}

fn warning(root: &Path, project: &ConfigProject, setup: &VitestSetupDependency) -> Warning {
    let config = config_path(root, project, setup);
    let declaration = relative_path(root, &setup.declaration_path);
    let project = project_name(project);
    let (r#type, message) = if let Some(specifier) = &setup.specifier {
        (
            "vitest-setup-unresolved",
            format!(
                "Vitest `{}` for project `{project}` in `{declaration}` at line {} could not resolve `{specifier}`.",
                setup.field.as_str(),
                setup.declaration_line,
            ),
        )
    } else {
        (
            "vitest-setup-dynamic",
            format!(
                "Vitest `{}` for project `{project}` in `{declaration}` at line {} is dynamic.",
                setup.field.as_str(),
                setup.declaration_line,
            ),
        )
    };
    Warning {
        r#type: r#type.to_string(),
        message,
        file: config,
        line: Some(setup.declaration_line),
    }
}

fn trigger_paths(
    root: &Path,
    project: &ConfigProject,
    setup: &VitestSetupDependency,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
) -> Vec<PathBuf> {
    let config = config_absolute_path(root, project, setup);
    let declaration = normalize(&setup.declaration_path);
    // A configured runner scope remains authoritative. Explicit policies
    // deliberately clear it, so those retain the parsed setup root instead.
    let scope = project
        .scope
        .as_deref()
        .map(|scope| normalize(&root.join(scope)))
        .unwrap_or_else(|| normalize(&setup.resolution_base));
    let mut triggered = BTreeSet::new();
    for changed in changed_files.iter().chain(deleted_files) {
        let changed = normalize(changed);
        let config_or_helper = changed == config
            || changed == declaration
            || setup.trigger_paths.iter().any(|trigger| {
                let trigger = normalize(trigger);
                changed == trigger
                    || (setup.specifier.is_none()
                        && trigger.is_dir()
                        && changed.starts_with(trigger))
            });
        let dynamic_owner_scope =
            setup.specifier.is_none() && (changed == scope || changed.starts_with(&scope));
        if config_or_helper || dynamic_owner_scope {
            triggered.insert(changed);
        }
    }
    triggered.into_iter().collect()
}

fn owner_tests(root: &Path, project: &ConfigProject, discovered: &DiscoveredTests) -> Vec<PathBuf> {
    let scope_filter = project
        .runner_project_arg
        .is_none()
        .then(|| {
            no_mistakes::codebase::test_discovery::ProjectTestFilter::from_project_ref(project).ok()
        })
        .flatten();
    discovered
        .tests
        .iter()
        .filter(|test| {
            if project.runner_project_arg.is_none() {
                let Some(filter) = &scope_filter else {
                    return false;
                };
                let relative = no_mistakes::codebase::ts_source::relative_slash_path(root, test);
                if !filter.is_match(&relative) {
                    return false;
                }
                return discovered
                    .targets_by_path
                    .get(*test)
                    .is_none_or(|targets| unnamed_project_owns_target(project, targets));
            }
            discovered
                .targets_by_path
                .get(*test)
                .is_some_and(|targets| {
                    targets.iter().any(|target| {
                        target.runner == "vitest"
                            && target.config == project.config
                            && target.project == project.runner_project_arg
                    })
                })
        })
        .cloned()
        .collect()
}

fn unnamed_project_owns_target(
    project: &ConfigProject,
    targets: &[no_mistakes::codebase::test_discovery::TestExecutionTarget],
) -> bool {
    let targets = targets
        .iter()
        .filter(|target| target.runner == "vitest")
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return false;
    }
    if !targets
        .iter()
        .any(|target| target.config.is_some() || target.project.is_some())
    {
        return true;
    }
    targets
        .iter()
        .any(|target| target.config == project.config && target.project.is_none())
}

fn owner_is_known(project: &ConfigProject) -> bool {
    project.config.is_some()
        || project.runner_project_arg.is_some()
        || project.policy_name.is_some()
        || project.scope.is_some()
}

fn config_path(root: &Path, project: &ConfigProject, setup: &VitestSetupDependency) -> String {
    relative_path(root, &config_absolute_path(root, project, setup))
}

fn config_absolute_path(
    root: &Path,
    project: &ConfigProject,
    setup: &VitestSetupDependency,
) -> PathBuf {
    project
        .config
        .as_deref()
        .map(|config| normalize(&root.join(config)))
        .unwrap_or_else(|| normalize(&setup.declaration_path))
}

fn project_name(project: &ConfigProject) -> &str {
    project
        .runner_project_arg
        .as_deref()
        .or(project.policy_name.as_deref())
        .unwrap_or("default")
}

fn normalize(path: &Path) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(path)
}

#[cfg(test)]
mod tests;
