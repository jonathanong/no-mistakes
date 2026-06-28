use super::*;

fn project(
    name: &str,
    include: &[&str],
    exclude: &[&str],
    config: Option<&str>,
    runner: Option<&str>,
    scope: Option<&str>,
) -> ConfigProject {
    ConfigProject {
        config: config.map(str::to_string),
        policy_name: Some(name.to_string()),
        runner_project_arg: runner.map(str::to_string),
        scope: scope.map(str::to_string),
        include: include.iter().map(|item| item.to_string()).collect(),
        exclude: exclude.iter().map(|item| item.to_string()).collect(),
    }
}

#[test]
fn merge_pushes_new_project_when_no_policy_name_matches() {
    let mut projects = vec![project(
        "unit",
        &["src/**/*.test.ts"],
        &[],
        None,
        None,
        None,
    )];

    merge_explicit_project(
        &mut projects,
        project("integration", &["tests/**/*.ts"], &[], None, None, None),
    );

    assert_eq!(projects.len(), 2);
    assert_eq!(projects[1].policy_name.as_deref(), Some("integration"));
}

#[test]
fn merge_preserves_existing_globs_when_explicit_project_omits_them() {
    let mut projects = vec![project(
        "unit",
        &["src/**/*.test.ts"],
        &["src/generated/**"],
        Some("vitest.config.mts"),
        None,
        Some("src"),
    )];

    merge_explicit_project(
        &mut projects,
        project("unit", &[], &[], None, Some("unit"), None),
    );

    assert_eq!(projects[0].include, vec!["src/**/*.test.ts"]);
    assert_eq!(projects[0].exclude, vec!["src/generated/**"]);
    assert_eq!(projects[0].config.as_deref(), Some("vitest.config.mts"));
    assert_eq!(projects[0].runner_project_arg.as_deref(), Some("unit"));
    assert_eq!(projects[0].scope.as_deref(), Some("src"));
}

#[test]
fn merge_overlays_non_empty_explicit_fields() {
    let mut projects = vec![project(
        "unit",
        &["old/**"],
        &["old-ignore/**"],
        None,
        None,
        None,
    )];

    merge_explicit_project(
        &mut projects,
        project(
            "unit",
            &["new/**"],
            &["new-ignore/**"],
            Some("new.config.mts"),
            Some("new-unit"),
            Some("new"),
        ),
    );

    assert_eq!(projects[0].include, vec!["new/**"]);
    assert_eq!(projects[0].exclude, vec!["new-ignore/**"]);
    assert_eq!(projects[0].config.as_deref(), Some("new.config.mts"));
    assert_eq!(projects[0].runner_project_arg.as_deref(), Some("new-unit"));
    assert_eq!(projects[0].scope.as_deref(), Some("new"));
}
