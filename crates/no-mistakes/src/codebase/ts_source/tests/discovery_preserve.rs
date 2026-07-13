use super::{fixture, git_add_all, git_init, write};
use tempfile::TempDir;

#[test]
fn discover_files_preserving_roots_walks_preserved_skip_dir_subtrees() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "fixtures/app/src/lib.rs", "");
    write(dir.path(), "fixtures/other/src/lib.rs", "");

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
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "fixtures/app/src/lib.rs", "");
    git_add_all(dir.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(dir.path());

    // This file is intentionally added after discovery. Both filtered views
    // must reuse the same request snapshot instead of rediscovering it.
    write(dir.path(), "fixtures/app/src/late.rs", "");
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
