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
    assert_eq!(
        exclusive.exclusive_rust_candidates(),
        std::slice::from_ref(&rust_file)
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

    let overlapping = NoMistakesConfig {
        rules: vec![
            rust_rule,
            RuleDef {
                rule: super::super::NO_EMPTY_OR_COMMENTS_ONLY_FILES.to_string(),
                scope: Some(RuleScope::Repository),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let shared = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &overlapping,
        &files,
        &files,
        &files,
        None,
    );
    assert!(shared.exclusive_rust_candidates().is_empty());
}

#[test]
fn dispatch_prepares_one_index_and_only_reads_preclassified_views() {
    let dispatch = include_str!("../../filesystem_dispatch.rs");

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

    assert_eq!(index.candidates(BANNED_PATHS), [tracked]);
    assert_eq!(
        index.candidates(super::super::NO_EMPTY_OR_COMMENTS_ONLY_FILES),
        files
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
