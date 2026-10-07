use super::*;
use crate::codebase::ts_source::VisiblePathSnapshot;
use std::sync::Arc;

fn boundary_fixture() -> TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/ts-source/discovery-git-boundary-cache"),
    )
}

fn assert_sibling_scope_reuse(root: &Path, repo: &Path) {
    let alpha = repo.join("packages/alpha");
    let beta = repo.join("packages/beta");
    git_init(repo);
    crate::test_support::git_add_force(repo, &["packages/alpha"]);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = VisiblePathSnapshot::new_observed(root, Some(observer.clone()));
    let stores = std::thread::scope(|scope| {
        [&alpha, &beta, &alpha, &beta]
            .into_iter()
            .map(|root| scope.spawn(|| snapshot.source_store_for(root)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(stores.iter().all(|store| Arc::ptr_eq(store, &stores[0])));
    assert_eq!(observer.snapshot().work["discovery.roots"], 2);
    assert_eq!(
        snapshot.paths_for(&alpha).as_slice(),
        [alpha.join("src/a.ts")]
    );
    assert_eq!(
        snapshot.paths_for(&beta).as_slice(),
        [beta.join("src/b.ts")]
    );
    assert_eq!(
        snapshot.tracked_paths_for(&alpha).as_slice(),
        [alpha.join("src/a.ts")]
    );
    assert!(snapshot.tracked_paths_for(&beta).is_empty());
    let projected = std::thread::scope(|scope| {
        [&alpha, &beta, &alpha, &beta]
            .into_iter()
            .map(|root| {
                scope.spawn(|| {
                    (
                        snapshot.paths_for(root),
                        snapshot.tracked_paths_for(root),
                        snapshot.git_index_paths_for(root),
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    for (index, root) in [&alpha, &beta].into_iter().enumerate() {
        let first = &projected[index];
        let concurrent = &projected[index + 2];
        // Pointer identity protects reuse, even for empty tracked projections.
        assert!(Arc::ptr_eq(&first.0, &concurrent.0));
        assert!(Arc::ptr_eq(&first.1, &concurrent.1));
        assert!(Arc::ptr_eq(&first.2, &concurrent.2));
        assert!(Arc::ptr_eq(&first.0, &snapshot.paths_for(root)));
        assert!(Arc::ptr_eq(&first.1, &snapshot.tracked_paths_for(root)));
        assert!(Arc::ptr_eq(&first.2, &snapshot.git_index_paths_for(root)));
    }
    assert_eq!(observer.snapshot().work["discovery.projections"], 2);
    assert_eq!(projected[0].2.as_slice(), [alpha.join("src/a.ts")]);
    assert!(projected[1].2.is_empty());
    // Another project must use the same frozen index, even after staging changes.
    crate::test_support::git_add_force(repo, &["packages/beta"]);
    assert!(snapshot.tracked_paths_for(&beta).is_empty());
    assert!(Arc::ptr_eq(
        &projected[1].2,
        &snapshot.git_index_paths_for(&beta)
    ));
    assert_eq!(observer.snapshot().work["discovery.projections"], 2);
    assert_eq!(observer.snapshot().work["discovery.roots"], 2);
    assert_eq!(
        VisiblePathSnapshot::new(root)
            .tracked_paths_for(&beta)
            .as_slice(),
        [beta.join("src/b.ts")]
    );
}

#[test]
fn nested_sibling_projects_share_one_frozen_git_boundary() {
    let fixture = boundary_fixture();
    assert_sibling_scope_reuse(fixture.path(), &fixture.path().join("repo"));
}

#[test]
fn external_sibling_projects_share_one_bounded_git_boundary() {
    let fixture = boundary_fixture();
    assert_sibling_scope_reuse(
        &fixture.path().join("request"),
        &fixture.path().join("repo"),
    );
}

#[test]
fn shared_outer_git_ancestors_do_not_expand_configured_scopes() {
    let fixture = boundary_fixture();
    git_init(fixture.path());
    let root = fixture.path().join("request");
    let alpha = fixture.path().join("repo/packages/alpha");
    assert_eq!(
        crate::codebase::ts_source::snapshot_scope_root(&root, &alpha),
        alpha
    );
    let snapshot = VisiblePathSnapshot::new(&root);
    let paths = snapshot.source_store_for(&alpha).inventory().paths();
    assert_eq!(paths.as_slice(), [alpha.join("src/a.ts")]);
}

#[test]
fn non_git_descendants_reuse_the_request_view_without_new_discovery() {
    let fixture = boundary_fixture();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = VisiblePathSnapshot::new_observed(fixture.path(), Some(observer.clone()));
    let request = snapshot.source_store_for(fixture.path());
    let nested = snapshot.source_store_for(&fixture.path().join("repo/packages/alpha"));
    assert!(Arc::ptr_eq(&request, &nested));
    assert_eq!(observer.snapshot().work["discovery.roots"], 1);
    assert_eq!(
        crate::codebase::ts_source::snapshot_scope_root(fixture.path(), fixture.path()),
        fixture.path()
    );
}

#[test]
fn prepared_scope_does_not_switch_views_when_git_metadata_changes() {
    let fixture = boundary_fixture();
    let root = fixture.path();
    let repo = root.join("repo");
    let alpha = repo.join("packages/alpha");
    let snapshot = VisiblePathSnapshot::new(root);
    let first = snapshot.source_store_for(&alpha);
    // A late Git boundary must not replace this scope's already prepared view.
    git_init(&repo);
    assert!(Arc::ptr_eq(&first, &snapshot.source_store_for(&alpha)));
    let prepared = VisiblePathSnapshot::new(root);
    let git_view = prepared.source_store_for(&alpha);
    std::fs::remove_dir_all(repo.join(".git")).unwrap();
    assert!(Arc::ptr_eq(&git_view, &prepared.source_store_for(&alpha)));
    assert!(!Arc::ptr_eq(&first, &git_view));
}
