//! Conservative impact handling for Vitest setup declarations that cannot be
//! resolved statically. These declarations deliberately stay out of the
//! ordinary dependency graph, but must not produce a confidently empty plan.

use crate::integration_tests::types::{ConfigProject, VitestSetupDependency};
use crate::tests::configured_plan_candidates::{merge_selected, selected_from_paths, stable_take};
use crate::tests::plan::relative_path;
use crate::tests::{SelectedTest, Warning};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn warnings(root: &Path, projects: Option<&[ConfigProject]>) -> Vec<Warning> {
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

/// Select a conservative, project-bounded fallback only after an unsafe setup
/// declaration is relevant to this change. `used` is applied before the global
/// plan limit so ordinary candidates retain their established priority.
pub(super) fn selection(
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

    for project in projects {
        for setup in project.vitest_setup.iter().filter(|setup| is_unsafe(setup)) {
            let triggers = trigger_paths(root, project, setup, changed_files, deleted_files);
            if triggers.is_empty() {
                continue;
            }
            triggered = true;
            let owner_tests = owner_tests(project, discovered);
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
    let reason = match (selected_owner, selected_framework) {
        (true, false) => {
            "Vitest setup dependencies could not be resolved statically; selected owning project tests"
        }
        (false, true) => {
            "Vitest setup dependencies could not be resolved statically; selected discovered Vitest tests"
        }
        (true, true) => {
            "Vitest setup dependencies could not be resolved statically; selected owning project and discovered Vitest tests"
        }
        (false, false) => unreachable!("a relevant Vitest setup fallback has an owner decision"),
    };
    Some((
        reason.to_string(),
        stable_take(candidates.into_values().collect(), limit),
    ))
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
    let literal_candidates = setup_literal_candidates(setup);
    let scope = project
        .scope
        .as_deref()
        .map(|scope| normalize(&root.join(scope)));
    let mut triggered = BTreeSet::new();
    for changed in changed_files.iter().chain(deleted_files) {
        let changed = normalize(changed);
        let config_or_helper = changed == config
            || changed == declaration
            || setup
                .trigger_paths
                .iter()
                .any(|trigger| changed == normalize(trigger));
        let literal_candidate = setup.specifier.is_some() && literal_candidates.contains(&changed);
        let dynamic_owner_scope = setup.specifier.is_none()
            && scope
                .as_ref()
                .is_some_and(|scope| changed == *scope || changed.starts_with(scope));
        if config_or_helper || literal_candidate || dynamic_owner_scope {
            triggered.insert(changed);
        }
    }
    triggered.into_iter().collect()
}

fn owner_tests(project: &ConfigProject, discovered: &DiscoveredTests) -> Vec<PathBuf> {
    discovered
        .tests
        .iter()
        .filter(|test| {
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

fn owner_is_known(project: &ConfigProject) -> bool {
    project.config.is_some()
        || project.runner_project_arg.is_some()
        || project.policy_name.is_some()
        || project.scope.is_some()
}

fn setup_literal_candidates(setup: &VitestSetupDependency) -> BTreeSet<PathBuf> {
    let Some(specifier) = setup.specifier.as_deref() else {
        return BTreeSet::new();
    };
    let specifier_path = Path::new(specifier);
    if !specifier_path.is_absolute() && !specifier.starts_with('.') {
        return BTreeSet::new();
    }
    let base = if specifier_path.is_absolute() {
        specifier_path.to_path_buf()
    } else {
        setup.resolution_base.join(specifier_path)
    };
    let mut candidates = BTreeSet::from([normalize(&base)]);
    for extension in ["ts", "mts", "cts", "js", "mjs", "cjs"] {
        candidates.insert(normalize(&base.with_extension(extension)));
        candidates.insert(normalize(&base.join(format!("index.{extension}"))));
    }
    candidates
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
mod tests {
    use super::*;
    use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
    use no_mistakes::codebase::test_discovery::TestExecutionTarget;
    use std::collections::BTreeMap;

    fn project(scope: Option<&str>, setup: VitestSetupDependency) -> ConfigProject {
        ConfigProject {
            config: Some("vitest.config.ts".to_string()),
            policy_name: None,
            runner_project_arg: Some("unit".to_string()),
            scope: scope.map(str::to_string),
            include: Vec::new(),
            exclude: Vec::new(),
            vitest_setup: vec![setup],
        }
    }

    fn setup(specifier: Option<&str>) -> VitestSetupDependency {
        VitestSetupDependency {
            field: VitestSetupField::SetupFiles,
            specifier: specifier.map(str::to_string),
            resolved_path: None,
            resolution_base: PathBuf::from("/repo/config"),
            declaration_path: PathBuf::from("/repo/config/setup.ts"),
            declaration_line: 7,
            trigger_paths: BTreeSet::from([PathBuf::from("/repo/config/setup.ts")]),
        }
    }

    fn discovered() -> DiscoveredTests {
        let unit = PathBuf::from("/repo/unit/a.test.ts");
        let other = PathBuf::from("/repo/other/b.test.ts");
        let target = |project: Option<&str>| TestExecutionTarget {
            runner: "vitest".to_string(),
            config: Some("vitest.config.ts".to_string()),
            project: project.map(str::to_string),
            base_command: Vec::new(),
            runner_args: Vec::new(),
        };
        DiscoveredTests {
            tests: vec![unit.clone(), other.clone()],
            targets_by_path: BTreeMap::from([
                (unit, vec![target(Some("unit"))]),
                (other, vec![target(Some("other"))]),
            ]),
            used_fallback: false,
        }
    }

    #[test]
    fn dynamic_setup_falls_back_to_its_owner_scope() {
        let root = Path::new("/repo");
        let result = selection(
            root,
            &[root.join("config/setup.ts")],
            &[],
            Some(&[project(Some("unit"), setup(None))]),
            &discovered(),
            &HashSet::new(),
            10,
        )
        .expect("owner-scope change must be conservative");
        assert!(result.0.contains("owning project"));
        assert_eq!(
            result
                .1
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["unit/a.test.ts"]
        );
    }

    #[test]
    fn unresolved_literal_matches_deleted_extension_candidate() {
        let root = Path::new("/repo");
        let result = selection(
            root,
            &[],
            &[root.join("config/missing.ts")],
            Some(&[project(Some("unit"), setup(Some("./missing")))]),
            &discovered(),
            &HashSet::new(),
            10,
        )
        .expect("deleted literal candidate must trigger fallback");
        assert_eq!(result.1.len(), 1);
        assert_eq!(result.1[0].test_file, "unit/a.test.ts");
    }

    #[test]
    fn dynamic_setup_tracks_imported_helper_outside_owner_scope() {
        let root = Path::new("/repo");
        let mut dynamic = setup(None);
        dynamic
            .trigger_paths
            .insert(root.join("config/helpers/resolve-setup.ts"));
        let project = project(Some("unit"), dynamic);
        let result = selection(
            root,
            &[root.join("config/helpers/resolve-setup.ts")],
            &[],
            Some(std::slice::from_ref(&project)),
            &discovered(),
            &HashSet::new(),
            10,
        )
        .expect("a dynamic setup helper must trigger the owning fallback");
        assert_eq!(result.1.len(), 1);
        assert_eq!(result.1[0].test_file, "unit/a.test.ts");

        let deleted = selection(
            root,
            &[],
            &[root.join("config/helpers/resolve-setup.ts")],
            Some(std::slice::from_ref(&project)),
            &discovered(),
            &HashSet::new(),
            10,
        )
        .expect("a deleted dynamic setup helper must trigger the owning fallback");
        assert_eq!(deleted.1.len(), 1);
        assert_eq!(deleted.1[0].test_file, "unit/a.test.ts");
    }

    #[test]
    fn known_owner_without_eligible_tests_does_not_widen_to_framework() {
        let root = Path::new("/repo");
        let mut discovered = discovered();
        discovered.targets_by_path.clear();
        let result = selection(
            root,
            &[root.join("unit/src/service.ts")],
            &[],
            Some(&[project(Some("unit"), setup(None))]),
            &discovered,
            &HashSet::new(),
            10,
        )
        .expect("known owner-scope change must still record fallback");
        assert!(result.0.contains("owning project"));
        assert!(result.1.is_empty(), "{result:#?}");
    }

    #[test]
    fn missing_owner_identity_uses_framework_fallback() {
        let root = Path::new("/repo");
        let mut unknown = project(Some("unit"), setup(None));
        unknown.config = None;
        unknown.runner_project_arg = None;
        unknown.policy_name = None;
        unknown.scope = None;
        let result = selection(
            root,
            &[root.join("config/setup.ts")],
            &[],
            Some(&[unknown]),
            &discovered(),
            &HashSet::new(),
            10,
        )
        .expect("unknown ownership should use framework fallback");
        assert!(result.0.contains("discovered Vitest tests"));
        assert_eq!(result.1.len(), 2);
    }

    #[test]
    fn warnings_name_the_config_field_project_and_location() {
        let root = Path::new("/repo");
        let warnings = warnings(root, Some(&[project(Some("unit"), setup(None))]));
        assert_eq!(warnings[0].r#type, "vitest-setup-dynamic");
        assert_eq!(warnings[0].file, "vitest.config.ts");
        assert!(warnings[0].message.contains("`setupFiles`"));
        assert!(warnings[0].message.contains("project `unit`"));
        assert!(warnings[0].message.contains("config/setup.ts"));
    }
}
