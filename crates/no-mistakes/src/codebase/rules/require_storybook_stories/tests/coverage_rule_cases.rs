use super::*;

#[test]
fn namespace_and_allow_findings_cover_non_matching_edges() {
    let root = normalize_path(&fixture("missing"));
    let project_root = root.as_path();
    let story = normalize_path(&root.join("stories/card.stories.tsx"));
    let resolver = empty_resolver(&root);
    let mut shared = CheckFactMap {
        files: vec![root.join("components/Card.tsx")],
        ts: HashMap::from([(
            story.clone(),
            CheckFileFacts {
                storybook: Some(StorybookFileFacts {
                    used_runtime_imports: vec![
                        crate::codebase::storybook::UsedRuntimeImport {
                            source: "../../Outside".to_string(),
                            imported: "*".to_string(),
                            local: "Outside".to_string(),
                            namespace: true,
                            line: 7,
                        },
                        crate::codebase::storybook::UsedRuntimeImport {
                            source: "./missing".to_string(),
                            imported: "*".to_string(),
                            local: "MissingNamespace".to_string(),
                            namespace: true,
                            line: 9,
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
    let story_files = [story, root.join("stories/missing-facts.stories.tsx")]
        .into_iter()
        .collect();

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
    let allow_files = types::GlobMatcher::new(opts.allow_files.keys()).unwrap();
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
fn config_helpers_cover_tsconfig_and_storybook_fallbacks() {
    let root = normalize_path(&fixture("covered"));
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        Some(&root.join("tsconfig.json")),
        &root,
        &visible,
    )
    .unwrap();
    assert_eq!(tsconfig.dir, root);
    let discovered =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    assert_eq!(discovered.dir, root);

    let mut missing = crate::config::v2::NoMistakesConfig::default();
    missing.tests.storybook.configs = Some(StringOrList::One(".storybook/missing.ts".to_string()));
    let patterns =
        config::effective_story_patterns(&root, &root, &missing, &types::Options::default());
    assert_eq!(patterns, vec!["**/*.stories.{ts,tsx,js,jsx}"]);

    let story_root = fixture("defaults");
    let config_path = story_root.join(".storybook/main.ts");
    let mut absolute = crate::config::v2::NoMistakesConfig::default();
    absolute.tests.storybook.configs =
        Some(StringOrList::One(config_path.to_string_lossy().to_string()));
    let patterns = config::effective_story_patterns(
        &story_root,
        &story_root,
        &absolute,
        &types::Options::default(),
    );
    assert_eq!(patterns, vec!["storybook/**/*.stories.tsx"]);

    let fallback_root = fixture("single-story-config");
    let mut root_relative = crate::config::v2::NoMistakesConfig::default();
    root_relative.tests.storybook.configs =
        Some(StringOrList::One(".storybook/main.ts".to_string()));
    let patterns = config::effective_story_patterns(
        &fallback_root,
        &fallback_root.join("web"),
        &root_relative,
        &types::Options::default(),
    );
    assert_eq!(
        patterns,
        vec![format!(
            "{}/custom/**/*.examples.tsx",
            normalize_path(&fallback_root).to_string_lossy()
        )]
    );
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
fn storybook_config_paths_prefer_project_root() {
    let root = fixture("config-project-root");
    let mut cfg = config_with_project_root(
        "web",
        r#"
include_all_react_named_exports: true
"#,
    );
    cfg.tests.storybook.configs = Some(StringOrList::One(".storybook/main.ts".to_string()));
    let findings = check(&root, &cfg, None).unwrap();

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
    assert!(findings[0]
        .message
        .contains("add an accepted colocated test"));
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Missing.tsx#Missing")
    );
}

#[test]
fn colocated_tests_can_cover_selected_components() {
    let root = fixture("colocated-tests");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
allow_colocated_tests: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(
        findings
            .iter()
            .filter_map(|finding| finding.target.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "components/NestedOnly.tsx#NestedOnly",
            "components/SpecOnly.tsx#SpecOnly",
        ]
    );
}
