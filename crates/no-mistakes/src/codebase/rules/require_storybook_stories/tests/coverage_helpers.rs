use super::*;

#[test]
fn coverage_helpers_handle_unresolved_and_reexport_edges() {
    let root = normalize_path(&fixture("missing"));
    let story = normalize_path(&root.join("stories/card.stories.tsx"));
    let linked_story = normalize_path(&root.join("stories/linked.story.tsx"));
    let component = normalize_path(&root.join("components/Card.tsx"));
    let reexport = normalize_path(&root.join("components/index.ts"));
    let cycle = normalize_path(&root.join("components/cycle.ts"));
    let resolver = empty_resolver(&root);
    let stories = vec!["stories/card.stories.tsx".to_string()];
    let matcher = types::GlobMatcher::new(&stories).unwrap();
    let component_key = types::component_key("components/Card.tsx", "Card");
    let component_keys = HashSet::from([component_key.clone()]);
    let shared = CheckFactMap {
        files: vec![story.clone(), linked_story.clone()],
        ts: HashMap::from([
            (
                story.clone(),
                CheckFileFacts {
                    storybook: Some(StorybookFileFacts {
                        used_runtime_imports: vec![
                            crate::codebase::storybook::UsedRuntimeImport {
                                source: "./missing".to_string(),
                                imported: "Missing".to_string(),
                                local: "Missing".to_string(),
                                namespace: false,
                                line: 2,
                            },
                            crate::codebase::storybook::UsedRuntimeImport {
                                source: "../components/index".to_string(),
                                imported: "CardAlias".to_string(),
                                local: "CardAlias".to_string(),
                                namespace: false,
                                line: 3,
                            },
                            crate::codebase::storybook::UsedRuntimeImport {
                                source: "../components/index".to_string(),
                                imported: "CardAlias".to_string(),
                                local: "CardAliasNamespace".to_string(),
                                namespace: true,
                                line: 4,
                            },
                            crate::codebase::storybook::UsedRuntimeImport {
                                source: "../components/index".to_string(),
                                imported: "*".to_string(),
                                local: "Components".to_string(),
                                namespace: false,
                                line: 5,
                            },
                            crate::codebase::storybook::UsedRuntimeImport {
                                source: "../../Outside".to_string(),
                                imported: "Outside".to_string(),
                                local: "Outside".to_string(),
                                namespace: false,
                                line: 6,
                            },
                        ],
                        side_effect_imports: vec![
                            crate::codebase::storybook::StorybookSideEffectImport {
                                source: "./missing-side-effect".to_string(),
                                line: 6,
                            },
                            crate::codebase::storybook::StorybookSideEffectImport {
                                source: "./linked.story".to_string(),
                                line: 7,
                            },
                            crate::codebase::storybook::StorybookSideEffectImport {
                                source: "./card.stories".to_string(),
                                line: 8,
                            },
                        ],
                    }),
                    ..Default::default()
                },
            ),
            (
                linked_story,
                CheckFileFacts {
                    storybook: Some(StorybookFileFacts::default()),
                    ..Default::default()
                },
            ),
            (
                component,
                CheckFileFacts {
                    symbols: Some(std::sync::Arc::new(FileSymbols {
                        exports: vec![Export {
                            name: "Card".to_string(),
                            local: None,
                            kind: ExportKind::Function,
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    })),
                    ..Default::default()
                },
            ),
            (
                reexport.clone(),
                CheckFileFacts {
                    symbols: Some(std::sync::Arc::new(FileSymbols {
                        exports: vec![
                            Export {
                                name: "TypeOnly".to_string(),
                                local: None,
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "Card".to_string(),
                                },
                                line: 1,
                                is_type_only: true,
                            },
                            Export {
                                name: "Other".to_string(),
                                local: None,
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "Card".to_string(),
                                },
                                line: 2,
                                is_type_only: false,
                            },
                            Export {
                                name: "CardAlias".to_string(),
                                local: None,
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "Card".to_string(),
                                },
                                line: 3,
                                is_type_only: false,
                            },
                            Export {
                                name: "*".to_string(),
                                local: None,
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "*".to_string(),
                                },
                                line: 4,
                                is_type_only: false,
                            },
                        ],
                        imports: Vec::new(),
                    })),
                    ..Default::default()
                },
            ),
            (
                cycle.clone(),
                CheckFileFacts {
                    symbols: Some(std::sync::Arc::new(FileSymbols {
                        exports: vec![Export {
                            name: "Cycle".to_string(),
                            local: None,
                            kind: ExportKind::ReExport {
                                source: "./cycle".to_string(),
                                imported: "Cycle".to_string(),
                            },
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    })),
                    ..Default::default()
                },
            ),
        ])
        .into_iter()
        .map(|(path, facts)| (path, std::sync::Arc::new(facts)))
        .collect(),
        ..Default::default()
    };

    let story_files =
        coverage::reachable_story_files(&root, &shared, &matcher, &resolver, &component_keys);
    assert!(story_files.contains(&story));
    assert!(story_files.iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "linked.story.tsx")
    }));

    let covered = coverage::directly_covered_components(
        &root,
        &shared,
        &story_files
            .iter()
            .cloned()
            .chain([root.join("stories/missing-facts.stories.tsx")])
            .collect(),
        &resolver,
        &component_keys,
    );
    assert_eq!(covered, [component_key].into_iter().collect());

    let cycle_story = CheckFactMap {
        files: vec![story.clone()],
        ts: HashMap::from([
            (
                story.clone(),
                CheckFileFacts {
                    storybook: Some(StorybookFileFacts {
                        used_runtime_imports: vec![crate::codebase::storybook::UsedRuntimeImport {
                            source: "../components/cycle".to_string(),
                            imported: "Cycle".to_string(),
                            local: "Cycle".to_string(),
                            namespace: false,
                            line: 1,
                        }],
                        side_effect_imports: Vec::new(),
                    }),
                    ..Default::default()
                },
            ),
            (
                cycle,
                CheckFileFacts {
                    symbols: Some(std::sync::Arc::new(FileSymbols {
                        exports: vec![Export {
                            name: "Cycle".to_string(),
                            local: None,
                            kind: ExportKind::ReExport {
                                source: "./cycle".to_string(),
                                imported: "Cycle".to_string(),
                            },
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    })),
                    ..Default::default()
                },
            ),
        ])
        .into_iter()
        .map(|(path, facts)| (path, std::sync::Arc::new(facts)))
        .collect(),
        ..Default::default()
    };
    assert!(coverage::directly_covered_components(
        &root,
        &cycle_story,
        &[story].into_iter().collect(),
        &resolver,
        &component_keys,
    )
    .is_empty());
}

#[test]
fn selection_and_transitive_helpers_cover_skip_paths() {
    let root = normalize_path(&fixture("missing"));
    let project_root = root.as_path();
    let included = root.join("components/Included.tsx");
    let excluded = root.join("components/Excluded.tsx");
    let parsed_bad = root.join("components/Bad.tsx");
    let missing_react = root.join("components/MissingReact.tsx");
    let no_source = root.join("components/NoSource.tsx");
    let outside = root.parent().unwrap().join("External.tsx");
    let opts = types::Options {
        include_all_react_named_exports: true,
        exclude: vec!["components/Excluded.tsx".to_string()],
        required_props: vec!["data-story".to_string()],
        ..Default::default()
    };
    let include = types::GlobMatcher::new(&opts.include).unwrap();
    let exclude = types::GlobMatcher::new(&opts.exclude).unwrap();
    let mut shared = CheckFactMap {
        files: vec![
            excluded.clone(),
            parsed_bad.clone(),
            missing_react.clone(),
            no_source.clone(),
            included.clone(),
        ],
        ts: HashMap::from([
            (
                outside,
                CheckFileFacts {
                    react: Some(react_facts(vec![react_component(
                        "External",
                        "External.tsx",
                        Vec::new(),
                    )])),
                    source: Some("export function External() { return null }".into()),
                    ..Default::default()
                },
            ),
            (
                excluded,
                CheckFileFacts {
                    react: Some(react_facts(vec![react_component(
                        "Excluded",
                        "components/Excluded.tsx",
                        Vec::new(),
                    )])),
                    source: Some("export function Excluded() { return null }".into()),
                    ..Default::default()
                },
            ),
            (
                parsed_bad,
                CheckFileFacts {
                    parse_error: Some("bad".to_string()),
                    source: Some("export function Bad() { return null }".into()),
                    ..Default::default()
                },
            ),
            (
                missing_react,
                CheckFileFacts {
                    source: Some("export function MissingReact() { return null }".into()),
                    ..Default::default()
                },
            ),
            (
                no_source.clone(),
                CheckFileFacts {
                    react: Some(react_facts(vec![react_component(
                        "NoSource",
                        "components/NoSource.tsx",
                        vec![ComponentRef {
                            name: "Leaf".to_string(),
                            file: "components/Leaf.tsx".to_string(),
                        }],
                    )])),
                    symbols: Some(std::sync::Arc::new(FileSymbols {
                        exports: vec![Export {
                            name: "NoSource".to_string(),
                            local: None,
                            kind: ExportKind::Function,
                            line: 3,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    })),
                    ..Default::default()
                },
            ),
            (
                included.clone(),
                CheckFileFacts {
                    react: Some(react_facts(vec![react_component(
                        "Included",
                        "components/Included.tsx",
                        Vec::new(),
                    )])),
                    source: Some(
                        "export function Included() { return <div data-story=\"yes\" /> }".into(),
                    ),
                    symbols: Some(std::sync::Arc::new(FileSymbols {
                        exports: vec![Export {
                            name: "Included".to_string(),
                            local: None,
                            kind: ExportKind::Function,
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    })),
                    ..Default::default()
                },
            ),
        ])
        .into_iter()
        .map(|(path, facts)| (path, std::sync::Arc::new(facts)))
        .collect(),
        ..Default::default()
    };
    let test_filter = crate::codebase::test_filter::TestFileFilter::new(
        &root,
        &crate::config::v2::NoMistakesConfig::default(),
    );

    let selected = selection::selected_components(
        &root,
        project_root,
        &shared,
        &opts,
        &include,
        &exclude,
        &test_filter,
    );
    assert_eq!(
        selected
            .iter()
            .map(|component| component.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "components/Included.tsx#Included",
            "components/NoSource.tsx#NoSource",
        ]
    );

    std::sync::Arc::get_mut(shared.ts.get_mut(&no_source).unwrap())
        .unwrap()
        .react = Some(react_facts(vec![react_component(
        "NoSource",
        "components/NoSource.tsx",
        Vec::new(),
    )]));
    let covered = coverage_graph::transitive_covered_components(
        &root,
        project_root,
        &shared,
        &[
            "components/NoSource.tsx#NoSource".to_string(),
            "components/Unknown.tsx#Unknown".to_string(),
        ]
        .into_iter()
        .collect(),
        &HashSet::new(),
    );
    assert_eq!(
        covered,
        [
            "components/NoSource.tsx#NoSource".to_string(),
            "components/Unknown.tsx#Unknown".to_string(),
        ]
        .into_iter()
        .collect()
    );

    let resolver = empty_resolver(project_root);
    let same_project = root.join("components/SameProject.tsx");
    let other_project = root.parent().unwrap().join("OtherProject.tsx");
    let boundary_shared = CheckFactMap {
        ts: HashMap::from([
            (
                root.join("loader.ts"),
                CheckFileFacts {
                    dynamic_imports: Some(
                        crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::TestFacts {
                            dynamic_imports: vec![crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::DynamicImport {
                                specifier: Some("./components/SameProject".to_string()),
                                line: 1,
                            }, crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::DynamicImport {
                                specifier: Some("./components/MissingDynamic".to_string()),
                                line: 2,
                            }],
                            mock_specifiers: vec!["../OtherProject".to_string()],
                        },
                    ),
                    ..Default::default()
                },
            ),
            (
                root.parent().unwrap().join("loader.ts"),
                CheckFileFacts {
                    dynamic_imports: Some(
                        crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::TestFacts {
                            dynamic_imports: vec![crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::DynamicImport {
                                specifier: Some("./OtherProject".to_string()),
                                line: 1,
                            }],
                            mock_specifiers: Vec::new(),
                        },
                    ),
                    ..Default::default()
                },
            ),
        ])
            .into_iter()
            .map(|(path, facts)| (path, std::sync::Arc::new(facts)))
            .collect(),
        ..Default::default()
    };
    let boundary_files =
        coverage_graph::dynamic_or_mock_boundary_files(project_root, &boundary_shared, &resolver);
    assert!(boundary_files.contains(&normalize_path(&same_project)));
    assert!(!boundary_files.contains(&normalize_path(&other_project)));
}
