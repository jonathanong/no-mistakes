use super::*;

#[test]
fn scoped_states_ignore_targets_without_a_matching_scope() {
    let root = fixture("paths");
    let targets = vec![root.join("lost.md")];
    let sources = super::super::super::source_store_for_files(&targets);
    let (states, names) = scoped_states(
        &root,
        &[root.join("docs")],
        &targets,
        &targets,
        ScopeOptions {
            roots: &BTreeSet::from(["CLAUDE.md".to_string()]),
            indexes: &BTreeSet::from(["README.md".to_string()]),
            max_depth: DEFAULT_MAX_DEPTH,
            sources: &sources,
        },
    )
    .unwrap();
    assert!(states.is_empty());
    assert!(names.is_empty());
}

#[test]
fn nested_project_uses_its_own_scope_not_a_reachable_outer_document() {
    let root = fixture("nested-scope");
    let mut config = config("", &["**/*.md"], &[]);
    config.projects.insert(
        "nested-docs".to_string(),
        crate::config::v2::schema::Project {
            root: Some("docs".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].projects = vec!["nested-docs".to_string()];

    let findings = run(&root, &config, &["CLAUDE.md", "docs/lost.md"]).unwrap();

    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["docs/lost.md"],
        "the outer CLAUDE.md must not make a nested project document reachable"
    );
}

#[test]
fn nested_and_repository_scope_graphs_are_partitioned_by_most_specific_owner() {
    let root = fixture("scope-partition");
    let mut config = config("", &["**/*.md"], &[]);
    config.projects.insert(
        "nested-docs".to_string(),
        crate::config::v2::schema::Project {
            root: Some("docs".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].projects = vec!["nested-docs".to_string()];

    let findings = run(
        &root,
        &config,
        &[
            "CLAUDE.md",
            "outer-linked.md",
            "outer.md",
            "docs/CLAUDE.md",
            "docs/nested-linked.md",
            "docs/nested.md",
        ],
    )
    .unwrap();

    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["docs/nested.md", "outer.md"],
        "nested scope roots and links must not make outer files reachable, and repository roots and links must not make nested files reachable"
    );
}

#[test]
fn canonical_aliases_never_cross_scope_ownership() {
    let root = fixture("scope-alias");
    let mut config = config("", &["**/*.md"], &[]);
    config.projects.insert(
        "nested-docs".to_string(),
        crate::config::v2::schema::Project {
            root: Some("docs".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].projects = vec!["nested-docs".to_string()];

    let findings = run(
        &root,
        &config,
        &["CLAUDE.md", "outer.md", "docs/CLAUDE.md", "docs/alias.md"],
    )
    .unwrap();

    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["outer.md"],
        "a nested canonical alias must stay in its nested scope and not make the outer file reachable"
    );
}

#[test]
fn rule_application_remappers_ignore_aliases_owned_by_other_applications() {
    let root = fixture("scope-application-remapper");
    let mut config = config("", &["**/*.md"], &[]);
    config.projects.insert(
        "docs".to_string(),
        crate::config::v2::schema::Project {
            root: Some("docs".to_string()),
            ..Default::default()
        },
    );
    config.projects.insert(
        "other".to_string(),
        crate::config::v2::schema::Project {
            root: Some("other".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["docs".to_string()];
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: RULE_ID.to_string(),
        include: vec!["**/*.md".to_string()],
        options: serde_yaml::from_str("rootFilenames: [alias.md]").unwrap(),
        projects: vec!["other".to_string()],
        ..Default::default()
    });

    let findings = run(
        &root,
        &config,
        &["docs/CLAUDE.md", "docs/guide.md", "other/alias.md"],
    )
    .unwrap();

    assert!(
        findings.is_empty(),
        "an alias selected only by another rule application must not make this application's case-resolved link ambiguous"
    );
}

#[test]
fn excluded_markdown_remains_in_the_selected_scope_graph() {
    let root = fixture("filtered-target-graph");
    let findings = run(
        &root,
        &config("", &["guide.md"], &[]),
        &["CLAUDE.md", "guide.md"],
    )
    .unwrap();

    assert!(
        findings.is_empty(),
        "an excluded CLAUDE.md still supplies a discovery edge for an included target"
    );
}

#[test]
fn scope_roots_choose_the_deepest_root_and_deduplicate_exact_ties() {
    let root = Path::new("/repo");
    let mut config = config("", &[], &[]);
    config.projects.insert(
        "same-root".to_string(),
        crate::config::v2::schema::Project {
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    config.projects.insert(
        "nested".to_string(),
        crate::config::v2::schema::Project {
            root: Some("docs".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].projects = vec!["same-root".to_string(), "nested".to_string()];

    let scopes = super::super::super::markdown_scope::scope_roots(root, &config, &config.rules[0]);
    assert_eq!(scopes, vec![root.join("docs"), root.to_path_buf()]);
    assert_eq!(
        super::super::super::markdown_scope::scope_root_for_path(
            &scopes,
            &root.join("docs/file.md")
        ),
        Some(&root.join("docs"))
    );
}
