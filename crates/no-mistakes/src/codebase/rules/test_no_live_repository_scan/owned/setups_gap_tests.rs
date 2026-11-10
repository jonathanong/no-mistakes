use super::{select, SetupGap};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::rules::PreparedVitestProjectCatalog;
use crate::codebase::ts_resolver::normalize_path;
use crate::integration_tests::types::{ConfigProject, VitestSetupDependency, VitestSetupField};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn catalog(
    projects: Result<Vec<ConfigProject>, CheckInputFailure>,
) -> PreparedVitestProjectCatalog {
    PreparedVitestProjectCatalog::from_load_result(projects)
}

fn project(config: Option<&str>, include: &str) -> ConfigProject {
    ConfigProject {
        config: config.map(str::to_string),
        workspace: false,
        policy_name: None,
        runner_project_arg: None,
        scope: None,
        include: vec![include.to_string()],
        exclude: Vec::new(),
        matcher_warnings: Vec::new(),
        vitest_setup: Vec::new(),
        declared_deadlines: Default::default(),
    }
}

fn setup(
    field: VitestSetupField,
    resolved: Option<&str>,
    declaration: &str,
    line: u32,
    extends: bool,
) -> VitestSetupDependency {
    VitestSetupDependency {
        field,
        specifier: None,
        needs_final_catalog_reparse: false,
        unresolved_config_extends: None,
        config_extends_provenance: extends,
        resolved_path: resolved.map(PathBuf::from),
        resolution_base: PathBuf::from("/repo"),
        declaration_path: PathBuf::from(declaration),
        declaration_line: line,
        trigger_paths: BTreeSet::new(),
        resolver_candidate_paths: BTreeSet::new(),
        conservative_specifiers: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
    }
}

fn root_file() -> PathBuf {
    PathBuf::from("/repo/src/fn.mts")
}

fn expect_unreadable(result: Result<Vec<super::super::Project>, SetupGap>) -> PathBuf {
    match result {
        Err(SetupGap::Unreadable(file)) => file,
        Err(SetupGap::Unresolved(_)) => panic!("unresolved setup"),
        Ok(_) => panic!("selected project"),
    }
}

#[test]
fn catalog_error_names_that_config() {
    let failure = CheckInputFailure::source("/repo/vitest.custom.ts", "broken");
    let file = expect_unreadable(select(&catalog(Err(failure)), Path::new("/repo"), &[]));
    assert_eq!(file, normalize_path(Path::new("/repo/vitest.custom.ts")));
}

#[test]
fn catalog_error_without_a_file_uses_vitest_config() {
    let failure = CheckInputFailure {
        file: None,
        property: None,
        message: "broken".to_string(),
    };
    let file = expect_unreadable(select(&catalog(Err(failure)), Path::new("/repo"), &[]));
    assert_eq!(
        file,
        normalize_path(&Path::new("/repo").join("vitest.config.ts"))
    );
}

#[test]
fn config_failures_keep_the_earliest_path_over_setup_holes() {
    let middle = project(Some("m.ts"), "[");
    let mut earliest = project(Some("a.ts"), "**/*.mts");
    earliest.matcher_warnings.push("dynamic".to_string());
    let mut later = project(Some("z.ts"), "**/*.mts");
    later.matcher_warnings.push("other".to_string());
    let mut hole = project(Some("hole.ts"), "**/*.mts");
    hole.vitest_setup.push(setup(
        VitestSetupField::SetupFiles,
        None,
        "/repo/b.ts",
        3,
        false,
    ));
    let file = expect_unreadable(select(
        &catalog(Ok(vec![middle, earliest, later, hole])),
        Path::new("/repo"),
        &[root_file()],
    ));
    assert_eq!(file, normalize_path(&Path::new("/repo").join("a.ts")));
}

#[test]
fn no_projects_uses_vitest_config() {
    let file = expect_unreadable(select(
        &catalog(Ok(Vec::new())),
        Path::new("/repo"),
        &[root_file()],
    ));
    assert_eq!(
        file,
        normalize_path(&Path::new("/repo").join("vitest.config.ts"))
    );
}

#[test]
fn unmatched_project_names_its_config() {
    let file = expect_unreadable(select(
        &catalog(Ok(vec![project(
            Some("vitest.config.ts"),
            "tests/**/*.mts",
        )])),
        Path::new("/repo"),
        &[root_file()],
    ));
    assert_eq!(
        file,
        normalize_path(&Path::new("/repo").join("vitest.config.ts"))
    );
}

#[test]
fn unresolved_setup_entries_are_sorted_and_deduped() {
    let mut project = project(Some("vitest.config.ts"), "**/*.mts");
    project.vitest_setup.extend([
        setup(VitestSetupField::SetupFiles, None, "/repo/b.ts", 0, false),
        setup(VitestSetupField::SetupFiles, None, "/repo/b.ts", 0, false),
        setup(VitestSetupField::SetupFiles, None, "/repo/a.ts", 4, false),
    ]);
    match select(
        &catalog(Ok(vec![project])),
        Path::new("/repo"),
        &[root_file()],
    ) {
        Err(SetupGap::Unresolved(holes)) => {
            assert_eq!(
                holes,
                vec![
                    (normalize_path(Path::new("/repo/a.ts")), 4),
                    (normalize_path(Path::new("/repo/b.ts")), 1),
                ]
            );
        }
        Err(SetupGap::Unreadable(_)) => panic!("unreadable config"),
        Ok(_) => panic!("selected project"),
    }
}

#[test]
fn resolved_setup_and_global_setup_select_the_project() {
    let mut project = project(Some("vitest.config.ts"), "**/*.mts");
    project.vitest_setup.extend([
        setup(
            VitestSetupField::SetupFiles,
            None,
            "/repo/extends.ts",
            2,
            true,
        ),
        setup(
            VitestSetupField::GlobalSetup,
            None,
            "/repo/global.ts",
            3,
            false,
        ),
        setup(
            VitestSetupField::GlobalSetup,
            Some("/repo/global.mts"),
            "/repo/global.mts",
            4,
            false,
        ),
        setup(
            VitestSetupField::SetupFiles,
            Some("/repo/setup.mts"),
            "/repo/setup.mts",
            5,
            false,
        ),
    ]);
    let selected = select(
        &catalog(Ok(vec![project])),
        Path::new("/repo"),
        &[root_file()],
    )
    .expect("selected");
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].setups,
        vec![normalize_path(Path::new("/repo/setup.mts"))]
    );
    assert_eq!(
        selected[0].global_setups,
        vec![normalize_path(Path::new("/repo/global.mts"))]
    );
    assert_eq!(
        selected[0].config.as_deref(),
        Some(normalize_path(&Path::new("/repo").join("vitest.config.ts")).as_path())
    );
}

#[test]
fn missing_config_path_still_selects() {
    let mut project = project(None, "**/*.mts");
    project.vitest_setup.push(setup(
        VitestSetupField::SetupFiles,
        Some("/repo/setup.mts"),
        "/repo/setup.mts",
        5,
        false,
    ));
    let selected = select(
        &catalog(Ok(vec![project])),
        Path::new("/repo"),
        &[root_file()],
    )
    .expect("selected");
    assert_eq!(selected.len(), 1);
    assert!(selected[0].config.is_none());
}
