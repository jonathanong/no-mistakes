use super::*;
use crate::config::v2::load_v2_config;
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/banned-paths/fixture")
            .join(name),
    )
}

#[test]
fn flags_configured_path_globs() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let config = load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![
        root.join("web/pages/index.tsx"),
        root.join("web/src/pages/account.tsx"),
        root.join("web/app/[topicType]/page.tsx"),
        root.join("web/app/topic/page.tsx"),
        root.join("web/app/t/page.tsx"),
        root.join("web/app/file.js"),
        root.join("web/app/file.ts"),
        root.join("web/app/file.css"),
        root.join("web/app/components/button.ts"),
        root.join("web/app/[[...slug]]/page.tsx"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    let files: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(
        files,
        vec![
            "web/app/[[...slug]]/page.tsx",
            "web/app/[topicType]/page.tsx",
            "web/app/file.js",
            "web/app/file.ts",
            "web/app/topic/page.tsx",
            "web/pages/index.tsx",
            "web/src/pages/account.tsx",
        ]
    );
    let default_message = findings
        .iter()
        .find(|finding| finding.file == "web/app/topic/page.tsx")
        .map(|finding| finding.message.as_str());
    assert_eq!(
        default_message,
        Some("web/app/topic/page.tsx: path is banned by `web/app/topic/**`")
    );
}

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
fn respects_rule_include_and_suppression() {
    let root = fixture_root("suppressed");
    let config_path = root.join(".no-mistakes.yml");
    let config = load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![
        root.join("web/pages/index.tsx"),
        root.join("web/pages/allowed.tsx"),
        root.join("other/pages/index.tsx"),
    ];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    super::super::suppress_rule_findings(&root, &mut findings);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "web/pages/index.tsx");
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
        external_root.display()
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
