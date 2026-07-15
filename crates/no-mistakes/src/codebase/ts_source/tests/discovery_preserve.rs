use super::{fixture, git_add_all, git_init};
use std::path::PathBuf;
use tempfile::TempDir;

fn saved_fixture(name: &str) -> TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ts-source")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn rebase_walk_path_preserves_paths_outside_the_walker_root() {
    let dir = saved_fixture("discovery-frozen-visible");
    let path = dir.path().join("src/main.mts");

    // Normal walker entries are always descendants of the walker root. This
    // unrelated root exercises the defensive fallback without changing files.
    assert_eq!(
        crate::codebase::ts_source::rebase_walk_path(
            dir.path(),
            dir.path().join("unrelated").as_path(),
            &path,
        ),
        path
    );
}

#[test]
fn discover_files_preserving_roots_walks_preserved_skip_dir_subtrees() {
    let dir = saved_fixture("discovery-preserved-roots");

    let files = crate::codebase::ts_source::discover_files_preserving_roots(
        dir.path(),
        &["fixtures".to_string()],
        &[dir.path().join("fixtures/app")],
    );

    assert_eq!(
        files,
        vec![
            dir.path().join("fixtures/app/src/lib.rs"),
            dir.path().join("src/main.mts"),
        ]
    );
}

#[test]
fn preserving_roots_from_visible_reuses_the_supplied_snapshot() {
    let dir = saved_fixture("discovery-frozen-visible");
    git_init(dir.path());
    git_add_all(dir.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(dir.path());

    // Runtime creation is the invariant under test: both filtered views must
    // reuse the earlier snapshot instead of rediscovering this late file.
    std::fs::write(dir.path().join("fixtures/app/src/late.rs"), "").unwrap();
    git_add_all(dir.path());
    for skip in [&["fixtures".to_string()][..], &[][..]] {
        let files = crate::codebase::ts_source::discover_files_preserving_roots_from_visible(
            dir.path(),
            skip,
            &[dir.path().join("fixtures/app")],
            &visible,
        );
        assert!(!files.contains(&dir.path().join("fixtures/app/src/late.rs")));
    }

    let fresh = crate::codebase::ts_source::discover_files_preserving_roots(
        dir.path(),
        &[],
        &[dir.path().join("fixtures/app")],
    );
    assert!(fresh.contains(&dir.path().join("fixtures/app/src/late.rs")));
}

#[test]
fn filtered_views_keep_files_deleted_after_the_snapshot() {
    let dir = saved_fixture("discovery-deleted");
    let path = dir.path().join("src/main.mts");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let visible = snapshot.paths_for(dir.path());
    std::fs::remove_file(&path).unwrap();

    // The frozen inventory remains authoritative. SourceStore owns the later
    // read failure and memoizes it for every consumer in this request.
    assert_eq!(
        crate::codebase::ts_source::discover_files_from_visible(dir.path(), &[], &visible),
        vec![path.clone()]
    );
    assert!(snapshot
        .classification_for(dir.path(), &path)
        .is_some_and(crate::codebase::ts_source::FileClassification::is_lexical_file));
}

#[test]
fn preserved_root_discovery_matches_non_git_hidden_file_semantics() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");
    let visible = crate::codebase::ts_source::discover_files(dir.path(), &[]);
    let preserved = crate::codebase::ts_source::discover_files_preserving_roots(
        dir.path(),
        &[],
        &[dir.path().join("visible/fixtures")],
    );
    let hidden = dir.path().join(".hidden/source.mts");

    assert!(visible.contains(&hidden));
    assert!(preserved.contains(&hidden));
}

#[test]
fn fallback_visible_discovery_prunes_git_metadata_but_keeps_other_hidden_paths() {
    let dir = saved_fixture("fallback-git-pruning");
    // A real `.git` tree cannot be stored as a repository fixture. Renaming
    // the saved pseudo-metadata directory is the runtime mutation under test.
    std::fs::rename(dir.path().join("git.fixture"), dir.path().join(".git")).unwrap();

    assert!(crate::codebase::ts_source::git_visible_files(dir.path()).is_none());
    let visible = crate::codebase::ts_source::discover_visible_paths(dir.path());

    assert!(visible.contains(&dir.path().join("visible.ts")));
    assert!(visible.contains(&dir.path().join(".hidden/source.mts")));
    assert!(!visible.contains(&dir.path().join(".git/objects/trap.ts")));
    assert!(visible
        .iter()
        .all(|path| !path.starts_with(dir.path().join(".git"))));
}

#[test]
fn nested_scope_adapters_filter_request_snapshot_supersets_to_the_requested_root() {
    let fixture = crate::test_support::materialize_gitignore_fixture("nested-scope-boundary");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());

    let alpha = fixture.path().join("packages/alpha");
    let beta_file = fixture.path().join("packages/beta/src/beta.ts");
    let root_file = fixture.path().join("root.ts");
    let generated = alpha.join("generated/keep.ts");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(fixture.path());
    let candidates = snapshot.paths_for(&alpha);

    // Same-repository nested roots intentionally reuse the request snapshot.
    assert!(candidates.contains(&beta_file));
    assert!(candidates.contains(&root_file));

    let scoped = crate::codebase::ts_source::discover_files_from_visible(
        &alpha,
        &["generated".to_string()],
        &candidates,
    );
    assert!(scoped.contains(&alpha.join("src/alpha.ts")));
    assert!(!scoped.contains(&generated));
    assert!(scoped.iter().all(|path| path.starts_with(&alpha)));

    let preserved = crate::codebase::ts_source::discover_files_preserving_roots_from_visible(
        &alpha,
        &["generated".to_string()],
        std::slice::from_ref(&alpha.join("generated")),
        &candidates,
    );
    assert!(preserved.contains(&generated));
    assert!(preserved.iter().all(|path| path.starts_with(&alpha)));
    assert!(!preserved.contains(&beta_file));
    assert!(!preserved.contains(&root_file));
}

#[test]
fn visible_snapshot_normalizes_discovery_paths_for_dot_component_roots() {
    let root = fixture("nextjs-selectors/frontend-tsconfig");
    let normalized_root = crate::codebase::ts_source::normalize_discovery_path(&root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let paths = snapshot.paths_for(&root);

    assert!(paths.contains(&normalized_root.join("web/app/page.tsx")));
    assert!(paths.iter().all(|path| path.starts_with(&normalized_root)));

    let candidate = root.join("web/app/page.tsx");
    let supplied = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
        &root,
        std::slice::from_ref(&candidate),
    );
    assert_eq!(
        supplied.paths_for(&root).as_slice(),
        [normalized_root.join("web/app/page.tsx")]
    );
}

#[test]
fn visible_snapshot_normalizes_additional_root_discovery_paths() {
    let request_root = fixture("nextjs-selectors/frontend-tsconfig");
    let additional_root = fixture("react-traits-components/bad-file");
    let normalized_additional =
        crate::codebase::ts_source::normalize_discovery_path(&additional_root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&request_root);
    let paths = snapshot.paths_for(&additional_root);

    assert!(paths.contains(&normalized_additional.join("app/components/Broken.tsx")));
    assert!(paths
        .iter()
        .all(|path| path.starts_with(&normalized_additional)));
}

#[test]
fn visible_snapshot_reuses_the_inventory_and_source_store_for_each_scope() {
    let request_root = fixture("nextjs-selectors/frontend-tsconfig");
    let nested_root = request_root.join("web");
    let additional_root = fixture("react-traits-components/bad-file");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&request_root);

    let request_store = snapshot.source_store_for(&request_root);
    let nested_store = snapshot.source_store_for(&nested_root);
    let additional_store = snapshot.source_store_for(&additional_root);
    let repeated_additional_store = snapshot.source_store_for(&additional_root);

    assert!(std::sync::Arc::ptr_eq(&request_store, &nested_store));
    assert!(std::sync::Arc::ptr_eq(
        &additional_store,
        &repeated_additional_store
    ));
    assert!(!std::sync::Arc::ptr_eq(&request_store, &additional_store));
    assert_eq!(
        snapshot.paths_for(&request_root),
        request_store.inventory().paths()
    );

    let source_path = crate::codebase::ts_source::normalize_discovery_path(
        &request_root.join("web/app/page.tsx"),
    );
    let additional_path = crate::codebase::ts_source::normalize_discovery_path(
        &additional_root.join("app/components/Broken.tsx"),
    );
    assert_eq!(
        snapshot.tracked_paths_from(&[source_path.clone(), additional_path.clone()]),
        [source_path.clone(), additional_path]
    );
    let first = request_store.read_path(&source_path).unwrap();
    let second = request_store.read_path(&source_path).unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
}

#[test]
fn visible_snapshot_initializes_one_scoped_store_across_threads() {
    let request_root = fixture("nextjs-selectors/frontend-tsconfig");
    let additional_root = fixture("react-traits-components/bad-file");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&request_root);

    let stores = std::thread::scope(|scope| {
        (0..16)
            .map(|_| scope.spawn(|| snapshot.source_store_for(&additional_root)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(stores
        .iter()
        .all(|store| std::sync::Arc::ptr_eq(store, &stores[0])));
    let normalized_additional =
        crate::codebase::ts_source::normalize_discovery_path(&additional_root);
    assert!(stores[0]
        .inventory()
        .paths()
        .contains(&normalized_additional.join("app/components/Broken.tsx")));
}

#[test]
fn visible_snapshot_classifies_nested_git_scope_once() {
    let dir = saved_fixture("discovery-nested-git");
    let nested = dir.path().join("packages/nested");
    // Git metadata is necessarily runtime-only; the source hierarchy is a
    // saved fixture so this test mutates only repository boundaries.
    git_init(&nested);
    git_init(dir.path());
    crate::test_support::git_add_force(dir.path(), &["root.ts"]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());

    let first = snapshot.source_store_for(&nested);
    let second = snapshot.source_store_for(&nested);
    let nested_file = nested.join("src/nested.ts");

    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert!(first
        .inventory()
        .classification_for_path(&nested_file)
        .is_some_and(crate::codebase::ts_source::FileClassification::is_lexical_file));
    assert!(snapshot
        .tracked_paths_from(std::slice::from_ref(&nested_file))
        .is_empty());
}

#[test]
fn visible_snapshot_tracks_only_existing_index_paths_inside_git() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/banned-paths-tracked-only");
    let dir = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_force(dir.path(), &["tracked.patch", "deleted.patch"]);
    std::fs::remove_file(dir.path().join("deleted.patch")).unwrap();
    std::fs::rename(
        dir.path().join("gitignore-after.fixture"),
        dir.path().join(".gitignore"),
    )
    .unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    let visible = snapshot.paths_for(dir.path());
    let candidates = vec![
        dir.path().join("tracked.patch"),
        dir.path().join("deleted.patch"),
        dir.path().join("untracked-visible.patch"),
        dir.path().join("untracked-ignored.patch"),
    ];

    assert!(visible.contains(&dir.path().join("tracked.patch")));
    assert!(visible.contains(&dir.path().join("untracked-visible.patch")));
    assert!(!visible.contains(&dir.path().join("deleted.patch")));
    assert!(!visible.contains(&dir.path().join("untracked-ignored.patch")));
    assert_eq!(
        snapshot.tracked_paths_from(&candidates),
        vec![dir.path().join("tracked.patch")]
    );
}

#[test]
fn visible_snapshot_treats_non_git_fallback_and_supplied_paths_as_authoritative() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/banned-paths-tracked-only");
    let dir = crate::test_support::materialize_saved_fixture(&source);
    std::fs::rename(
        dir.path().join("gitignore-after.fixture"),
        dir.path().join(".gitignore"),
    )
    .unwrap();
    let visible = dir.path().join("untracked-visible.patch");
    let ignored = dir.path().join("untracked-ignored.patch");

    let discovered = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());
    assert_eq!(
        discovered.tracked_paths_from(&[visible.clone(), ignored.clone()]),
        vec![visible.clone()]
    );

    let supplied = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
        dir.path(),
        &[visible.clone(), ignored.clone()],
    );
    assert_eq!(
        supplied.tracked_paths_from(&[ignored.clone(), visible.clone()]),
        vec![ignored, visible]
    );
}
