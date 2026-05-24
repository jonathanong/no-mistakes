use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig};
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols};
use crate::config::v2::schema::{Project, ProjectType, StringOrList};
use crate::react_traits::report::types::{ComponentFacts, ComponentRef, Environment};
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

fn react_component(name: &str, file: &str, children: Vec<ComponentRef>) -> ComponentFacts {
    ComponentFacts {
        name: name.to_string(),
        file: file.to_string(),
        environment: Environment::Client,
        has_state: false,
        has_props: false,
        passes_props: false,
        uses_memo: false,
        uses_context_provider: false,
        uses_suspense: false,
        fetches: Vec::new(),
        dependencies: Vec::new(),
        children,
        inherited_from_children: None,
    }
}

fn react_facts(
    components: Vec<ComponentFacts>,
) -> crate::react_traits::analyze::file::FileAnalysis {
    crate::react_traits::analyze::file::FileAnalysis { components }
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
    async () => [
	      "../components/**/*.story.tsx",
	      123,
	      `../examples/**/*.stories.tsx`,
	      { directory: "../cards", files: "**/*.case.tsx" },
	      { directory: "../spread", ...extra },
	      { directory: "../defaults" },
	      { ["directory"]: "../ignored" },
    ]
  ),
  parameters: {
    stories: ["../ignored/**/*.stories.tsx"],
  },
};
"#,
    );

    assert_eq!(
        patterns,
        vec![
            "../components/**/*.story.tsx",
            "../examples/**/*.stories.tsx",
            "../cards/**/*.case.tsx",
            "../spread/**/*.@(mdx|stories.@(js|jsx|mjs|ts|tsx))",
            "../defaults/**/*.@(mdx|stories.@(js|jsx|mjs|ts|tsx))",
        ]
    );

    let define_config_patterns = config::extract_storybook_story_patterns(
        r#"
const config = { stories: ["../config/**/*.stories.tsx"] };
const unrelated = { stories: ["../ignored/**/*.stories.tsx"] };
export default defineConfig(config);
"#,
    );
    assert_eq!(define_config_patterns, vec!["../config/**/*.stories.tsx"]);

    let export_identifier_patterns = config::extract_storybook_story_patterns(
        r#"
const config = { stories: ["../identifier/**/*.stories.tsx"] };
export default config;
"#,
    );
    assert_eq!(
        export_identifier_patterns,
        vec!["../identifier/**/*.stories.tsx"]
    );

    let direct_call_patterns = config::extract_storybook_story_patterns(
        r#"
export default defineConfig({ stories: ["../direct-call/**/*.stories.tsx"] });
"#,
    );
    assert_eq!(
        direct_call_patterns,
        vec!["../direct-call/**/*.stories.tsx"]
    );

    let parenthesized_patterns = config::extract_storybook_story_patterns(
        r#"
export default defineConfig(({ stories: ["../parenthesized/**/*.stories.tsx"] }));
"#,
    );
    assert_eq!(
        parenthesized_patterns,
        vec!["../parenthesized/**/*.stories.tsx"]
    );

    let parenthesized_export_patterns = config::extract_storybook_story_patterns(
        r#"
export default ({ stories: ["../parenthesized-export/**/*.stories.tsx"] });
"#,
    );
    assert_eq!(
        parenthesized_export_patterns,
        vec!["../parenthesized-export/**/*.stories.tsx"]
    );

    let alias_patterns = config::extract_storybook_story_patterns(
        r#"
const config = { stories: ["../alias/**/*.stories.tsx"] };
const alias = config;
export default alias;
"#,
    );
    assert_eq!(alias_patterns, vec!["../alias/**/*.stories.tsx"]);

    let call_binding_patterns = config::extract_storybook_story_patterns(
        r#"
const config = defineConfig({ stories: ["../call-binding/**/*.stories.tsx"] });
export default config;
"#,
    );
    assert_eq!(
        call_binding_patterns,
        vec!["../call-binding/**/*.stories.tsx"]
    );

    let parenthesized_binding_patterns = config::extract_storybook_story_patterns(
        r#"
const config = ({ stories: ["../parenthesized-binding/**/*.stories.tsx"] });
export default config;
"#,
    );
    assert_eq!(
        parenthesized_binding_patterns,
        vec!["../parenthesized-binding/**/*.stories.tsx"]
    );

    for source in [
        r#"const { config } = setup(); export default {};"#,
        r#"const config = 1; export default config;"#,
        r#"const a = b; const b = a; export default a;"#,
        r#"export default defineConfig("not-object");"#,
        r#"export default function config() {}"#,
        r#"const unrelated = { stories: ["../ignored/**/*.stories.tsx"] };"#,
        r#"export default { ...extra, stories() {}, ["stories"]: ["../ignored/**/*.stories.tsx"] };"#,
    ] {
        assert!(config::extract_storybook_story_patterns(source).is_empty());
    }

    let function_patterns = config::extract_storybook_story_patterns(
        r#"
export default {
  stories: async function stories() {
    const ignored = "../ignored/**/*.stories.tsx";
    doSetup();
    return { directory: "../function", files: "*.docs.tsx" };
  },
};
"#,
    );
    assert_eq!(function_patterns, vec!["../function/*.docs.tsx"]);

    let expression_patterns = config::extract_storybook_story_patterns(
        r#"
export default {
  stories: () => {
    ("../expression/**/*.stories.tsx");
  },
};
"#,
    );
    assert_eq!(expression_patterns, vec!["../expression/**/*.stories.tsx"]);

    assert!(config::extract_storybook_story_patterns("export default { stories: [").is_empty());
    assert!(config::extract_storybook_story_patterns(
        r#"
export default {
  stories: { files: "*.stories.tsx" },
};
"#
    )
    .is_empty());
    assert!(config::extract_storybook_story_patterns(
        r#"
export default {
  stories: makeStories(),
};
"#
    )
    .is_empty());
    assert!(config::extract_storybook_story_patterns(
        r#"
export default {
  stories: () => {},
};
"#
    )
    .is_empty());
    assert!(config::extract_storybook_story_patterns(
        r#"
export default {
  stories: () => {
    return;
  },
};
"#
    )
    .is_empty());
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

    let patterns = vec!["./components/**/*.tsx".to_string()];
    let matcher = types::GlobMatcher::new(&patterns).unwrap();
    assert!(matcher.is_match("components/Card.tsx"));
    assert!(!matcher.is_match("stories/Card.stories.tsx"));

    let invalid = vec!["[".to_string()];
    let error = types::GlobMatcher::new(&invalid).unwrap_err().to_string();
    assert!(error.contains("invalid Storybook coverage glob"));
}

#[test]
fn reachable_story_files_skip_unreadable_story_facts() {
    let root = normalize_path(&fixture("missing"));
    let story = normalize_path(&root.join("stories/card.stories.tsx"));
    let stories = vec!["stories/**/*.stories.tsx".to_string()];
    let matcher = types::GlobMatcher::new(&stories).unwrap();
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
    let files = coverage::reachable_story_files(
        &root,
        &parse_error_facts,
        &matcher,
        &resolver,
        &components,
    );
    assert_eq!(files, [story.clone()].into_iter().collect());

    parse_error_facts.ts.insert(
        story.clone(),
        CheckFileFacts {
            storybook: None,
            ..Default::default()
        },
    );
    let files = coverage::reachable_story_files(
        &root,
        &parse_error_facts,
        &matcher,
        &resolver,
        &components,
    );
    assert_eq!(files, [story.clone()].into_iter().collect());

    parse_error_facts.ts.clear();
    let files = coverage::reachable_story_files(
        &root,
        &parse_error_facts,
        &matcher,
        &resolver,
        &components,
    );
    assert_eq!(files, [story].into_iter().collect());
}

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
                    symbols: Some(FileSymbols {
                        exports: vec![Export {
                            name: "Card".to_string(),
                            kind: ExportKind::Function,
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    }),
                    ..Default::default()
                },
            ),
            (
                reexport.clone(),
                CheckFileFacts {
                    symbols: Some(FileSymbols {
                        exports: vec![
                            Export {
                                name: "TypeOnly".to_string(),
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "Card".to_string(),
                                },
                                line: 1,
                                is_type_only: true,
                            },
                            Export {
                                name: "Other".to_string(),
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "Card".to_string(),
                                },
                                line: 2,
                                is_type_only: false,
                            },
                            Export {
                                name: "CardAlias".to_string(),
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "Card".to_string(),
                                },
                                line: 3,
                                is_type_only: false,
                            },
                            Export {
                                name: "*".to_string(),
                                kind: ExportKind::ReExport {
                                    source: "./Card".to_string(),
                                    imported: "*".to_string(),
                                },
                                line: 4,
                                is_type_only: false,
                            },
                        ],
                        imports: Vec::new(),
                    }),
                    ..Default::default()
                },
            ),
            (
                cycle.clone(),
                CheckFileFacts {
                    symbols: Some(FileSymbols {
                        exports: vec![Export {
                            name: "Cycle".to_string(),
                            kind: ExportKind::ReExport {
                                source: "./cycle".to_string(),
                                imported: "Cycle".to_string(),
                            },
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    }),
                    ..Default::default()
                },
            ),
        ]),
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
                    symbols: Some(FileSymbols {
                        exports: vec![Export {
                            name: "Cycle".to_string(),
                            kind: ExportKind::ReExport {
                                source: "./cycle".to_string(),
                                imported: "Cycle".to_string(),
                            },
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    }),
                    ..Default::default()
                },
            ),
        ]),
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
        ts: HashMap::from([
            (
                outside,
                CheckFileFacts {
                    react: Some(react_facts(vec![react_component(
                        "External",
                        "External.tsx",
                        Vec::new(),
                    )])),
                    source: Some("export function External() { return null }".to_string()),
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
                    source: Some("export function Excluded() { return null }".to_string()),
                    ..Default::default()
                },
            ),
            (
                parsed_bad,
                CheckFileFacts {
                    parse_error: Some("bad".to_string()),
                    source: Some("export function Bad() { return null }".to_string()),
                    ..Default::default()
                },
            ),
            (
                missing_react,
                CheckFileFacts {
                    source: Some("export function MissingReact() { return null }".to_string()),
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
                    symbols: Some(FileSymbols {
                        exports: vec![Export {
                            name: "NoSource".to_string(),
                            kind: ExportKind::Function,
                            line: 3,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    }),
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
                        "export function Included() { return <div data-story=\"yes\" /> }"
                            .to_string(),
                    ),
                    symbols: Some(FileSymbols {
                        exports: vec![Export {
                            name: "Included".to_string(),
                            kind: ExportKind::Function,
                            line: 1,
                            is_type_only: false,
                        }],
                        imports: Vec::new(),
                    }),
                    ..Default::default()
                },
            ),
        ]),
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

    shared.ts.get_mut(&no_source).unwrap().react = Some(react_facts(vec![react_component(
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
        ]),
        ..Default::default()
    };
    let boundary_files =
        coverage_graph::dynamic_or_mock_boundary_files(project_root, &boundary_shared, &resolver);
    assert!(boundary_files.contains(&normalize_path(&same_project)));
    assert!(!boundary_files.contains(&normalize_path(&other_project)));
}

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
    let root = fixture("covered");
    let tsconfig = config::resolve_tsconfig(&root, Some(&root.join("tsconfig.json"))).unwrap();
    assert_eq!(tsconfig.dir, root);
    let discovered = config::resolve_tsconfig(&root, None).unwrap();
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

#[test]
fn colocated_tests_do_not_cover_components_without_option() {
    let root = fixture("colocated-tests");
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

    assert_eq!(
        findings
            .iter()
            .filter_map(|finding| finding.target.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "components/MockTs.tsx#MockTs",
            "components/MockTsx.tsx#MockTsx",
            "components/NestedOnly.tsx#NestedOnly",
            "components/PlainTs.tsx#PlainTs",
            "components/PlainTsx.tsx#PlainTsx",
            "components/SpecOnly.tsx#SpecOnly",
        ]
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
