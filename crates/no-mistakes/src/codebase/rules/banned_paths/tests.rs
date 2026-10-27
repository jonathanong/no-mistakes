use super::*;
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/banned-paths/fixture")
            .join(name),
    )
}

// Output cases moved to cli_rra_snapshot_banned_paths. Keep bracket escaping,
// the absolute external-project skip bypass, and invalid-glob errors.

#[test]
fn escapes_only_literal_route_bracket_segments() {
    assert_eq!(
        escape_literal_route_brackets("web/app/[topicType]/**"),
        "web/app/[[]topicType[]]/**"
    );
    assert_eq!(
        escape_literal_route_brackets("web/app/[[...slug]]/**"),
        "web/app/[[][[]...slug[]][]]/**"
    );
    assert_eq!(
        escape_literal_route_brackets("web/app/*.[jt]s"),
        "web/app/*.[jt]s"
    );
    assert_eq!(
        escape_literal_route_brackets("web/app/[topicType].tsx"),
        "web/app/[[]topicType[]].tsx"
    );
    assert_eq!(
        escape_literal_route_brackets("web/app/[topicType"),
        "web/app/[topicType"
    );
}

#[test]
fn repository_application_bypasses_skips_but_project_application_does_not() {
    let root = fixture_root("fail");
    let external_root = root.parent().unwrap().join("external-project");
    let config: NoMistakesConfig = serde_yaml::from_str(&format!(
        r#"
projects:
  external:
    root: '{}'
rules:
  - rule: banned-paths
    scope: repository
    projects: [external]
    options:
      bannedPaths:
        - glob: build/repository.patch
        - glob: build/combined.patch
  - rule: banned-paths
    projects: [external]
    options:
      bannedPaths:
        - glob: build/project.patch
"#,
        external_root.display().to_string().replace('\'', "''")
    ))
    .unwrap();
    let files = vec![
        root.join("build/repository.patch"),
        external_root.join("build/combined.patch"),
        external_root.join("build/project.patch"),
    ];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "build/repository.patch");
}

#[test]
fn invalid_glob_patterns_are_reported() {
    let root = fixture_root("fail");
    let config: NoMistakesConfig = serde_yaml::from_str(
        r#"
rules:
  - rule: banned-paths
    scope: repository
    options:
      bannedPaths:
        - glob: "[a-"
"#,
    )
    .unwrap();
    let error = check_with_files(&root, &config, &[]).unwrap_err();
    assert!(error.to_string().contains("invalid glob"), "{error:#}");
}
