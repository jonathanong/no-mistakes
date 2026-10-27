use super::*;

// Output cases moved to cli_mrf_snapshot_markdown_reachability. Keep scope
// internals that check JSON cannot distinguish.

#[test]
fn scoped_states_ignore_targets_without_a_matching_scope() {
    let root = fixture("paths");
    let targets = vec![root.join("lost.md")];
    let sources = super::super::super::source_store_for_files(&targets);
    let mut plan = super::super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(targets.clone());
    let facts = super::super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    let (states, names) = scoped_states(
        &root,
        &[root.join("docs")],
        &targets,
        &targets,
        ScopeOptions {
            roots: &BTreeSet::from(["CLAUDE.md".to_string()]),
            indexes: &BTreeSet::from(["README.md".to_string()]),
            max_depth: DEFAULT_MAX_DEPTH,
            facts: &facts,
        },
    )
    .unwrap();
    assert!(states.is_empty());
    assert!(names.is_empty());
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
