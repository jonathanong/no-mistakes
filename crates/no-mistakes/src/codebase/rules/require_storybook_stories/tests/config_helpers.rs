use super::*;

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

    let direct_child = vec!["components/ui/*.tsx".to_string()];
    let matcher = types::GlobMatcher::new(&direct_child).unwrap();
    assert!(matcher.is_match("components/ui/Button.tsx"));
    assert!(!matcher.is_match("components/ui/sidebar/Context.tsx"));

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
