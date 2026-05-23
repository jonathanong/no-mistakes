use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig};
use crate::config::v2::schema::{Project, ProjectType, StringOrList};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/require-storybook-stories")
        .join(name)
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["web".to_string()],
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn config_with_storybook(options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.tests.storybook.configs = Some(StringOrList::One(".storybook/main.ts".to_string()));
    config
}

fn config_with_project_root(root: &str, options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.projects.get_mut("web").unwrap().root = Some(root.to_string());
    config
}

fn empty_resolver(root: &std::path::Path) -> ImportResolver<'static> {
    let tsconfig = Box::leak(Box::new(TsConfig {
        dir: root.to_path_buf(),
        paths: vec![],
        paths_dir: root.to_path_buf(),
        base_url: None,
    }));
    ImportResolver::new(tsconfig)
}

#[test]
fn rule_target_validation_and_unknown_projects_are_handled() {
    let root = fixture("missing");
    let mut invalid = config(
        r#"
stories: ["stories/**/*.stories.tsx"]
"#,
    );
    invalid.projects.insert(
        "api".to_string(),
        Project {
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    invalid.rules[0].projects = vec!["web".to_string(), "api".to_string()];

    let error = check(&root, &invalid, None).unwrap_err().to_string();
    assert!(error.contains("requires exactly one project target"));

    let mut unknown = config(
        r#"
stories: ["stories/**/*.stories.tsx"]
"#,
    );
    unknown.rules[0].projects = vec!["unknown".to_string()];

    let findings = check(&root, &unknown, None).unwrap();
    assert!(findings.is_empty());

    let facts = CheckFactMap::default();
    let direct = check_rule(&root, &unknown, &unknown.rules[0], &facts, None).unwrap();
    assert!(direct.is_empty());
}

#[test]
fn storybook_config_pattern_parser_handles_supported_shapes() {
    let patterns = config::extract_storybook_story_patterns(
        r#"
export default {
  stories: (
    [
      "../components/**/*.story.tsx",
      `../examples/**/*.stories.tsx`,
      { directory: "../cards", files: "**/*.case.tsx" },
      { directory: "../defaults" },
      { ["directory"]: "../ignored" },
    ]
  ),
};
"#,
    );

    assert_eq!(
        patterns,
        vec![
            "../components/**/*.story.tsx",
            "../examples/**/*.stories.tsx",
            "../cards/**/*.case.tsx",
            "../defaults/**/*.stories.@(js|jsx|mjs|ts|tsx)",
        ]
    );
}

#[test]
fn required_prop_and_glob_helpers_match_expected_inputs() {
    let opts = types::Options {
        required_props: vec!["data-track".to_string(), "aria-label".to_string()],
        ..Default::default()
    };

    assert!(types::source_has_required_prop(
        "<Card data-track=\"x\" />",
        &opts
    ));
    assert!(types::source_has_required_prop(
        "{'aria-label': label}",
        &opts
    ));
    assert!(!types::source_has_required_prop("<Card />", &opts));

    let patterns = vec!["./components/**/*.tsx".to_string(), "[".to_string()];
    let matcher = types::GlobMatcher::new(&patterns);
    assert!(matcher.is_match("components/Card.tsx"));
    assert!(!matcher.is_match("stories/Card.stories.tsx"));
}

#[test]
fn reachable_story_files_reports_story_fact_errors() {
    let root = normalize_path(&fixture("missing"));
    let story = normalize_path(&root.join("stories/card.stories.tsx"));
    let stories = vec!["stories/**/*.stories.tsx".to_string()];
    let matcher = types::GlobMatcher::new(&stories);
    let resolver = empty_resolver(&root);
    let components = HashSet::new();

    let mut parse_error_facts = CheckFactMap {
        files: vec![story.clone()],
        ts: HashMap::from([(
            story.clone(),
            CheckFileFacts {
                parse_error: Some("bad syntax".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let error = coverage::reachable_story_files(
        &root,
        &parse_error_facts,
        &matcher,
        &resolver,
        &components,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("failed to parse story file"));

    parse_error_facts.ts.insert(
        story.clone(),
        CheckFileFacts {
            storybook: None,
            ..Default::default()
        },
    );
    let error = coverage::reachable_story_files(
        &root,
        &parse_error_facts,
        &matcher,
        &resolver,
        &components,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("requires Storybook facts"));

    parse_error_facts.ts.clear();
    let files = coverage::reachable_story_files(
        &root,
        &parse_error_facts,
        &matcher,
        &resolver,
        &components,
    )
    .unwrap();
    assert_eq!(files, [story].into_iter().collect());
}

#[test]
fn namespace_and_allow_findings_cover_non_matching_edges() {
    let root = normalize_path(&fixture("missing"));
    let project_root = root.as_path();
    let story = normalize_path(&root.join("stories/card.stories.tsx"));
    let outside = root.parent().unwrap().join("outside.tsx");
    let resolver = empty_resolver(&root);
    let mut shared = CheckFactMap {
        files: vec![root.join("components/Card.tsx")],
        ts: HashMap::from([(
            story.clone(),
            CheckFileFacts {
                storybook: Some(StorybookFileFacts {
                    used_runtime_imports: vec![
                        crate::codebase::storybook::UsedRuntimeImport {
                            source: outside.to_string_lossy().to_string(),
                            imported: "*".to_string(),
                            local: "Outside".to_string(),
                            namespace: true,
                            line: 7,
                        },
                        crate::codebase::storybook::UsedRuntimeImport {
                            source: "../components/Card".to_string(),
                            imported: "*".to_string(),
                            local: "Cards".to_string(),
                            namespace: true,
                            line: 8,
                        },
                    ],
                    side_effect_imports: vec![],
                }),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let story_files = [story].into_iter().collect();

    let namespace_findings =
        findings::namespace_import_findings(&root, project_root, &shared, &story_files, &resolver);
    assert_eq!(namespace_findings.len(), 1);
    assert_eq!(namespace_findings[0].line, 8);

    let mut opts = types::Options::default();
    opts.allow_components
        .insert("components/Gone.tsx#Gone".to_string(), "gone".to_string());
    opts.allow_files.insert(
        "components/*.tsx".to_string(),
        "covered elsewhere".to_string(),
    );
    opts.allow_files.insert("blank".to_string(), "".to_string());
    opts.allow_files
        .insert("[".to_string(), "bad glob".to_string());
    let allow_files = types::GlobMatcher::new(opts.allow_files.keys());
    let findings = findings::stale_or_blank_allow_findings(
        &root,
        project_root,
        &opts,
        &HashSet::new(),
        &allow_files,
        &shared,
    );
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("does not match a selected component")));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "[" && finding.message.contains("does not match")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("must include a reason")));

    shared.files.clear();
}

#[test]
fn direct_and_transitive_story_coverage_passes() {
    let root = fixture("covered");
    let findings = check(
        &root,
        &config(
            r#"
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn project_without_root_uses_repository_root() {
    let root = fixture("covered");
    let mut cfg = config(
        r#"
include_all_react_named_exports: true
"#,
    );
    cfg.projects.get_mut("web").unwrap().root = None;

    let findings = check(&root, &cfg, None).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn default_story_config_and_test_exclusion_are_used() {
    let root = fixture("defaults");
    let findings = check(
        &root,
        &config_with_storybook(
            r#"
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn single_string_story_config_is_used() {
    let root = fixture("single-story-config");
    let findings = check(
        &root,
        &config_with_storybook(
            r#"
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn absolute_story_config_patterns_are_project_relative() {
    let root = fixture("defaults");
    let pattern = root.join("stories/**/*.stories.tsx");
    let relative = config::project_relative_pattern(
        &root,
        &root.join(".storybook"),
        &pattern.to_string_lossy(),
    );

    assert_eq!(relative, "stories/**/*.stories.tsx");
}

#[test]
fn imported_story_files_count_as_reachable_coverage() {
    let root = fixture("reachable-story-import");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/entry.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn side_effect_story_imports_extend_reachable_coverage() {
    let root = fixture("side-effect-story");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/entry.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn mdx_story_imports_count_as_story_coverage() {
    let root = fixture("mdx-story");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.mdx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn selected_leaves_are_covered_through_unselected_wrappers() {
    let root = fixture("unselected-wrapper");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include: ["components/Leaf.tsx"]
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn reports_selected_component_without_story() {
    let root = fixture("missing");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "components/Missing.tsx");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Missing.tsx#Missing")
    );
}

#[test]
fn same_file_siblings_are_not_implicitly_covered() {
    let root = fixture("same-file-sibling");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Card.tsx#Sibling")
    );
}

#[test]
fn helper_imports_do_not_count_as_direct_story_coverage() {
    let root = fixture("helper-import");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Hidden.tsx#Hidden")
    );
}

#[test]
fn story_imports_resolve_component_reexports() {
    let root = fixture("reexport");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn transitive_coverage_uses_project_relative_keys() {
    let root = fixture("project-root");
    let findings = check(
        &root,
        &config_with_project_root(
            "web",
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn component_and_file_opt_outs_use_no_mistakes_comments() {
    let root = fixture("comments");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn config_opt_outs_need_reasons_and_existing_targets() {
    let root = fixture("missing");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
allow_components:
  "components/Missing.tsx#Missing": ""
  "components/Gone.tsx#Gone": "no longer exists"
allow_files:
  "components/Card.tsx": "covered by story"
  "components/nope/**": "gone"
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("must include a reason")));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "components/nope/**"
            && finding.message.contains("does not match")));
}

#[test]
fn default_export_assignments_are_selected() {
    let root = fixture("default-export");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_default_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn dynamic_import_targets_are_not_required_by_include_all() {
    let root = fixture("dynamic");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}
