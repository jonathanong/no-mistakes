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
    let mut trigger_paths = BTreeSet::from([PathBuf::from("/repo/config/setup.ts")]);
    if specifier.is_some() {
        // Literal resolution candidates are retained on the parsed setup
        // dependency, including targets that no longer exist.
        trigger_paths.insert(PathBuf::from("/repo/config/missing.ts"));
    }
    VitestSetupDependency {
        field: VitestSetupField::SetupFiles,
        specifier: specifier.map(str::to_string),
        resolved_path: None,
        resolution_base: PathBuf::from("/repo/config"),
        declaration_path: PathBuf::from("/repo/config/setup.ts"),
        declaration_line: 7,
        trigger_paths,
        resolver_candidate_paths: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
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
fn mixed_unresolved_and_deleted_resolved_setup_reasons_are_truthful() {
    let root = Path::new("/repo");
    let mut resolved = setup(Some("./resolved"));
    resolved.resolved_path = Some(root.join("config/resolved.ts"));
    resolved
        .transitive_trigger_paths
        .insert(root.join("config/resolved-helper.ts"));
    let mut project = project(Some("unit"), setup(None));
    project.vitest_setup.push(resolved);
    let result = selection(
        root,
        &[root.join("config/setup.ts")],
        &[root.join("config/resolved-helper.ts")],
        Some(&[project]),
        &discovered(),
        &HashSet::new(),
        10,
    )
    .expect("both setup fallback triggers must select the owner scope");
    assert!(result.0.contains("could not be resolved statically"));
    assert!(result
        .0
        .contains("transitive dependency of a resolved setup was deleted"));
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
fn unnamed_vitest_owner_uses_its_include_scope() {
    let root = Path::new("/repo");
    let mut unnamed = project(Some("unit"), setup(None));
    unnamed.runner_project_arg = None;
    unnamed.include = vec!["unit/**/*.test.ts".to_string()];
    let mut discovered = discovered();
    for targets in discovered.targets_by_path.values_mut() {
        targets[0].project = None;
    }

    let result = selection(
        root,
        &[root.join("config/setup.ts")],
        &[],
        Some(&[unnamed]),
        &discovered,
        &HashSet::new(),
        10,
    )
    .expect("an unnamed owner still has a project filter");
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
fn unnamed_vitest_owner_does_not_cross_select_an_overlapping_config() {
    let root = Path::new("/repo");
    let mut unnamed = project(Some("unit"), setup(None));
    unnamed.config = Some("vitest.unit.config.ts".to_string());
    unnamed.runner_project_arg = None;
    unnamed.include = vec!["unit/**/*.test.ts".to_string()];
    let mut discovered = discovered();
    for targets in discovered.targets_by_path.values_mut() {
        targets[0].project = None;
        targets[0].config = Some("vitest.other.config.ts".to_string());
    }

    let result = selection(
        root,
        &[root.join("config/setup.ts")],
        &[],
        Some(&[unnamed]),
        &discovered,
        &HashSet::new(),
        10,
    )
    .expect("the setup still has a known owner");
    assert!(
        result.1.is_empty(),
        "wrong config target must not be selected"
    );
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
