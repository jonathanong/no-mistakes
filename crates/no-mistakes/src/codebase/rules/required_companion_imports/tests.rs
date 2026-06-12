use super::helpers::{source_dir_matches, split_dir_base};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/required-companion-imports")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn requires_companion_file_importing_rendered_specifier() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("src/components/Button.tsx"),
        root.join("src/components/Button.stories.tsx"),
        root.join("src/components/Card.tsx"),
        root.join("src/components/Card.stories.tsx"),
        root.join("src/components/Internal.tsx"),
        root.join("src/components/nested/Nested.tsx"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sourceDirs: [src/components]
directChildOnly: true
sourceExtensions: [.tsx]
excludeBasenames: [Internal.tsx]
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
stripSourcePrefix: src/
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/components/Card.tsx");
    assert!(findings[0].message.contains("@/components/Card"));
}

#[test]
fn source_names_are_escaped_before_building_companion_globs() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("src/components/[id].tsx"),
        root.join("src/components/[id].stories.tsx"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sourceExtensions: [.tsx]
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn commented_companion_imports_do_not_satisfy_requirement() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("src/components/Commented.tsx"),
        root.join("src/components/Commented.stories.tsx"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sourceExtensions: [.tsx]
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/components/Commented.tsx");
}

#[test]
fn typescript_companion_imports_are_parsed() {
    assert!(file_imports(
        &fixture_root("fixture"),
        "src/components/Plain.story.ts",
        "@/components/Plain"
    ));
}

#[test]
fn declaration_files_are_not_source_candidates() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/components/Button.d.ts")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn source_globs_and_prefix_excludes_can_narrow_candidates() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("src/components/Button.tsx"),
        root.join("src/components/Button.stories.tsx"),
        root.join("src/components/SkipWidget.tsx"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sourceGlobs: ["src/components/Button.tsx"]
sourceExtensions: [.tsx]
excludePrefixes: [Skip]
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn reports_when_no_companion_file_exists() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/components/Missing.tsx")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sourceExtensions: [.tsx]
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("no companion file found"));
}

#[test]
fn default_extensions_recursive_dirs_and_side_effect_imports_are_supported() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("src/components/nested/Nested.tsx"),
        root.join("src/components/nested/Nested.stories.tsx"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sourceDirs: [src/components]
excludeBasenames: [Nested.stories.tsx]
companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
specifierTemplate: "@/components/{sourceStem}"
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn helper_branches_cover_empty_dirs_missing_files_and_extension_normalization() {
    let opts = Options {
        source_extensions: vec!["mts".to_string()],
        ..Default::default()
    };

    assert!(source_extensions(&Options::default()).contains(".tsx"));
    assert!(source_extensions(&opts).contains(".mts"));
    assert!(!source_dir_matches("src/components", "", false));
    assert!(source_dir_matches(
        "src/components/nested",
        "src/components",
        false
    ));
    assert_eq!(
        split_dir_base("Root.ts"),
        (String::new(), "Root.ts".to_string())
    );
    assert!(!file_imports(
        &fixture_root("fixture"),
        "src/components/DoesNotExist.stories.tsx",
        "@/components/Missing"
    ));
}
