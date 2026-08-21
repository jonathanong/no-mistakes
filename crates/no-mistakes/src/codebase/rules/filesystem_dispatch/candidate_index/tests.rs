use super::*;
use crate::config::v2::schema::{Project, RuleDef, RuleScope};

fn fixture() -> (PathBuf, NoMistakesConfig, Arc<Vec<PathBuf>>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/filesystem-dispatch/forbidden-workspace-project-root/fixture",
    );
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    (root, config, files)
}

#[test]
fn classification_matches_legacy_rule_views_and_reuses_owned_results() {
    let (root, config, files) = fixture();
    let index =
        RuleCandidateIndex::prepare_with_inventory(&root, &config, &files, &files, &files, None);

    for rule_id in FILESYSTEM_RULE_IDS
        .iter()
        .copied()
        .filter(|rule_id| rule_enabled(&config, rule_id))
    {
        let preserved_roots = preserved::filesystem_rule_preserved_roots(&root, &config, rule_id);
        let skip = super::super::super::skip_dir_set(&config);
        let expected = files
            .iter()
            .filter(|path| {
                super::super::super::file_allowed_by_roots_and_skip(
                    &root,
                    &skip,
                    path,
                    &preserved_roots,
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(index.candidates(rule_id), expected, "{rule_id}");
    }

    let first = index
        .by_rule
        .get(FORBIDDEN_WORKSPACE_CLOSURE)
        .cloned()
        .expect("enabled rule is classified");
    let second = index
        .by_rule
        .get(FORBIDDEN_WORKSPACE_CLOSURE)
        .cloned()
        .expect("enabled rule is reused");
    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn rust_exclusivity_tracks_enabled_non_rust_candidate_overlap() {
    let root = crate::codebase::ts_resolver::normalize_path(Path::new(env!("CARGO_MANIFEST_DIR")));
    let rust_file = root.join("src/lib.rs");
    let files = Arc::new(vec![rust_file.clone()]);
    let rust_rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    };
    let path_specific_non_rust = NoMistakesConfig {
        rules: vec![
            rust_rule.clone(),
            RuleDef {
                rule: super::super::AGENTS_MD_MAX_SIZE.to_string(),
                scope: Some(RuleScope::Repository),
                ..Default::default()
            },
            RuleDef {
                rule: super::super::GITHUB_ACTIONS_PINNED_HASH.to_string(),
                scope: Some(RuleScope::Repository),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let exclusive = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &path_specific_non_rust,
        &files,
        &files,
        &files,
        Some(Arc::clone(&files)),
    );
    let agents = exclusive
        .by_rule
        .get(super::super::AGENTS_MD_MAX_SIZE)
        .cloned()
        .unwrap();
    let workflows = exclusive
        .by_rule
        .get(super::super::GITHUB_ACTIONS_PINNED_HASH)
        .cloned()
        .unwrap();
    assert!(Arc::ptr_eq(&agents, &workflows));
    assert!(Arc::ptr_eq(&agents, &files));
}

#[test]
fn dispatch_prepares_one_index_and_only_reads_preclassified_views() {
    let dispatch = concat!(
        include_str!("../../filesystem_dispatch.rs"),
        include_str!("../execute.rs"),
        include_str!("../execute/special.rs"),
    );

    assert_eq!(dispatch.matches("RuleCandidateIndex::prepare").count(), 1);
    assert_eq!(dispatch.matches("filesystem_rule_files(").count(), 0);
    assert!(dispatch.matches("candidates.candidates(").count() >= 3);
}

#[test]
fn classification_normalizes_deduplicates_and_keeps_metadata_rule_context() {
    let (root, config, _) = fixture();
    let package = root.join("fixtures/app/package.json");
    let metadata_context = root.join("packages/domain/package.json");
    let files = vec![
        root.join("fixtures/app/../app/package.json"),
        package.clone(),
    ];
    let metadata = vec![package.clone(), metadata_context.clone()];

    let index =
        RuleCandidateIndex::prepare_with_inventory(&root, &config, &files, &files, &metadata, None);
    let candidates = index.candidates(FORBIDDEN_WORKSPACE_CLOSURE);

    assert_eq!(
        candidates.iter().filter(|path| *path == &package).count(),
        1
    );
    assert!(candidates.contains(&metadata_context));
    assert!(candidates.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn banned_paths_uses_tracked_candidates_without_narrowing_other_rules() {
    let root = crate::codebase::ts_resolver::normalize_path(Path::new(env!("CARGO_MANIFEST_DIR")));
    let tracked = root.join("tracked.patch");
    let untracked = root.join("untracked.patch");
    let files = vec![tracked.clone(), untracked.clone()];
    let tracked_files = vec![tracked.clone()];
    let repository_rule = |rule: &str| RuleDef {
        rule: rule.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    };
    let config = NoMistakesConfig {
        rules: vec![
            repository_rule(BANNED_PATHS),
            repository_rule(super::super::VERSION_PIN_CONSISTENCY),
            repository_rule(super::super::NO_EMPTY_OR_COMMENTS_ONLY_FILES),
        ],
        ..Default::default()
    };

    let index = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &config,
        &files,
        &tracked_files,
        &[],
        None,
    );

    assert_eq!(
        index.candidates(BANNED_PATHS),
        std::slice::from_ref(&tracked)
    );
    assert_eq!(
        index.candidates(super::super::VERSION_PIN_CONSISTENCY),
        std::slice::from_ref(&tracked)
    );
    assert_eq!(
        index.candidates(super::super::NO_EMPTY_OR_COMMENTS_ONLY_FILES),
        files
    );
}

#[test]
fn markdown_repository_rules_use_the_full_tracked_inventory_not_untracked_files() {
    let root = crate::codebase::ts_resolver::normalize_path(Path::new(env!("CARGO_MANIFEST_DIR")));
    let tracked_root = root.join("CLAUDE.md");
    let tracked_doc = root.join("docs/tracked.md");
    let tracked_markdown = root.join("docs/tracked.markdown");
    let tracked_mdx = root.join("docs/tracked.mdx");
    let untracked_doc = root.join("docs/untracked.md");
    let files = vec![
        tracked_root.clone(),
        tracked_doc.clone(),
        tracked_markdown.clone(),
        tracked_mdx.clone(),
        untracked_doc,
    ];
    let tracked_files = vec![
        tracked_root.clone(),
        tracked_doc.clone(),
        tracked_markdown.clone(),
        tracked_mdx.clone(),
    ];
    let repository_rule = |rule: &str| RuleDef {
        rule: rule.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    };
    let config = NoMistakesConfig {
        rules: vec![
            repository_rule(super::super::MARKDOWN_MERMAID_VALIDATION),
            repository_rule(super::super::MARKDOWN_REACHABILITY),
            repository_rule(super::super::MARKDOWN_STRUCTURE_BUDGET),
        ],
        ..Default::default()
    };
    let inventory = Arc::new(tracked_files.clone());
    let index = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &config,
        &files,
        &tracked_files,
        &[],
        Some(inventory),
    );
    assert_eq!(
        index.candidates(super::super::MARKDOWN_MERMAID_VALIDATION),
        tracked_files
    );
    assert_eq!(
        index.candidates(super::super::MARKDOWN_REACHABILITY),
        tracked_files
    );
    assert_eq!(
        index.candidates(super::super::MARKDOWN_STRUCTURE_BUDGET),
        [tracked_root, tracked_doc, tracked_markdown, tracked_mdx]
    );
}

#[test]
fn markdown_inventory_keeps_external_project_docs_but_skips_generated_directories() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/markdown-external-project");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(&fixture.path().join("request"));
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new_observed(
        &root,
        Some(Arc::clone(&observer)),
    );
    let inventory = super::super::inventory::tracked_inventory_with_markdown_project_roots(
        &root, &config, &snapshot,
    );
    let index =
        RuleCandidateIndex::prepare_with_inventory(&root, &config, &[], &[], &[], Some(inventory));

    let external_docs = [
        fixture.path().join("external/CLAUDE.md"),
        fixture.path().join("external/escaped-suppressed.md"),
        fixture.path().join("external/guide.md"),
        fixture.path().join("external/suppressed-link.md"),
    ];
    let nested_request_docs = [
        fixture.path().join("request/docs/CLAUDE.md"),
        fixture.path().join("request/docs/guide.md"),
    ];
    let request_root_skips = [
        fixture.path().join("request/fixtures/ignored.md"),
        fixture.path().join("request/generated/ignored.md"),
    ];
    for rule_id in [MARKDOWN_REACHABILITY, MARKDOWN_STRUCTURE_BUDGET] {
        let candidates = index.candidates(rule_id);
        assert!(
            external_docs.iter().all(|path| candidates.contains(path)),
            "{rule_id} keeps the external tracked Markdown project: {candidates:?}"
        );
        assert!(
            nested_request_docs
                .iter()
                .all(|path| candidates.contains(path)),
            "{rule_id} keeps nested request-root projects outside skipped directories: {candidates:?}"
        );
        assert!(
            !candidates.contains(&fixture.path().join("external/generated/ignored.md"))
                && !candidates.contains(&fixture.path().join("external/coverage/ignored.md"))
                && request_root_skips
                    .iter()
                    .all(|path| !candidates.contains(path)),
            "{rule_id} excludes request-root, configured, and built-in skipped Markdown: {candidates:?}"
        );
    }

    let files = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    sources.read_path(&root.join("CLAUDE.md")).unwrap();
    let warmed_reads = sources.physical_read_count();
    let findings = super::super::run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        &root,
        &config,
        &files,
        super::super::PreparedFilesystemRuleInputs {
            snapshot: &snapshot,
            vitest_catalog: None,
            sources: Arc::clone(&sources),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        },
    )
    .unwrap();
    let pairs = findings
        .iter()
        .map(|finding| (finding.rule.as_str(), finding.file.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        pairs,
        [
            (MARKDOWN_REACHABILITY, "../external-two/guide.md"),
            (MARKDOWN_REACHABILITY, "../external/guide.md"),
            (MARKDOWN_REACHABILITY, "docs/guide.md"),
            (MARKDOWN_STRUCTURE_BUDGET, "../external/over-budget.md"),
            (MARKDOWN_STRUCTURE_BUDGET, "docs/over-budget.md"),
        ],
        "only external and non-skipped nested Markdown projects produce request-relative findings"
    );
    assert!(
        sources.physical_read_count() > warmed_reads,
        "external analysis uses the caller's warmed source store"
    );
    let observed_reads = observer.source_read_snapshot();
    assert!(
        observed_reads.contains_key(&root.join("CLAUDE.md"))
            && observed_reads.contains_key(&fixture.path().join("external/suppressed.md"))
            // This tracked symlink resolves inside its configured external
            // project and must retain the target's file-level suppression.
            && observed_reads.contains_key(&fixture.path().join("external/suppressed-link.md")),
        "external regular and symlink suppression reads stay attached to the caller's observer: {observed_reads:?}"
    );
}

#[test]
fn repository_banned_paths_uses_full_inventory_and_keeps_external_project_candidates() {
    let root = crate::codebase::ts_resolver::normalize_path(Path::new(env!("CARGO_MANIFEST_DIR")));
    let external_root = root.parent().unwrap().join("external-app");
    let source = root.join("src/lib.rs");
    let skipped = root.join("fixtures/generated.patch");
    let external = external_root.join("src/index.ts");
    let files = Arc::new(vec![source.clone(), external.clone()]);
    let inventory = Arc::new(vec![skipped.clone(), source.clone()]);
    let config = NoMistakesConfig {
        projects: [(
            "external".to_string(),
            Project {
                root: Some(external_root.to_string_lossy().into_owned()),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        rules: vec![
            RuleDef {
                rule: BANNED_PATHS.to_string(),
                scope: Some(RuleScope::Repository),
                ..Default::default()
            },
            RuleDef {
                rule: BANNED_PATHS.to_string(),
                projects: vec!["external".to_string()],
                ..Default::default()
            },
            RuleDef {
                rule: super::super::NO_EMPTY_OR_COMMENTS_ONLY_FILES.to_string(),
                scope: Some(RuleScope::Repository),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let index = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &config,
        &files,
        &files,
        &[],
        Some(inventory),
    );

    let mut expected_banned_paths = vec![skipped, source.clone(), external];
    expected_banned_paths.sort();
    assert_eq!(index.candidates(BANNED_PATHS), expected_banned_paths);
    assert_eq!(
        index.candidates(super::super::NO_EMPTY_OR_COMMENTS_ONLY_FILES),
        std::slice::from_ref(&source)
    );
}
