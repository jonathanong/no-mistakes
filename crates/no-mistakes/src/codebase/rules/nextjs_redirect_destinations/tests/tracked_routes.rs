use super::*;
use crate::codebase::ts_source::VisiblePathSnapshot;

fn tracked_fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("tracked-routes"));
    let root = fixture.path();
    std::fs::rename(root.join(".gitignore.fixture"), root.join(".gitignore")).unwrap();
    crate::test_support::git_init(root);
    crate::test_support::git_add_force(
        root,
        &[
            ".gitignore",
            ".no-mistakes.yml",
            ".filesystem.yml",
            "next.config.ts",
            "app/tracked",
            "app/(group)",
            "app/posts",
            "app/docs",
            "app/optional",
        ],
    );
    fixture
}

#[test]
fn tracked_routes_reject_literal_and_tuple_destinations_until_pages_are_staged() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let config = config("{trackedRoutesOnly: true}");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = VisiblePathSnapshot::new_observed(root, Some(observer.clone()));
    let files = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let discovery_count = observer.snapshot().work["discovery.roots"];
    let prepared =
        check_with_files_sources_and_snapshot(root, &config, &files, &sources, Some(&snapshot))
            .unwrap();
    let standalone = check(root, &config).unwrap();
    assert_eq!(prepared, standalone);
    assert_eq!(prepared.len(), 6, "{prepared:?}");
    assert_eq!(observer.snapshot().work["discovery.roots"], discovery_count);
    assert_eq!(
        serde_json::to_string(&prepared).unwrap(),
        serde_json::to_string(&check(root, &config).unwrap()).unwrap()
    );
    assert!(prepared
        .iter()
        .all(|finding| finding.message.contains("'/untracked")
            || finding.message.contains("'/ignored")));

    crate::test_support::git_add_force(root, &["app/untracked", "app/ignored"]);
    assert!(check(root, &config).unwrap().is_empty());
    // A prepared request keeps its original Git inventory even if the index changes.
    assert_eq!(
        check_with_files_sources_and_snapshot(root, &config, &files, &sources, Some(&snapshot))
            .unwrap(),
        prepared
    );
}

#[test]
fn filesystem_routes_remain_default_and_rewrites_can_be_disabled() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    assert!(!Options::default().tracked_routes_only);
    let default = check(root, &config("{}")).unwrap();
    assert_eq!(default.len(), 3, "{default:?}");
    assert!(default
        .iter()
        .all(|finding| finding.message.contains("'/ignored")));
    let redirects = check(
        root,
        &config("{trackedRoutesOnly: true, includeRewrites: false}"),
    )
    .unwrap();
    assert_eq!(redirects.len(), 4);
    assert!(redirects
        .iter()
        .all(|finding| finding.message.contains("redirect destination")));
}

#[test]
fn tracked_routes_require_boolean_option_and_real_prepared_git_inventory() {
    let project = tracked_fixture();
    let root = project.path();
    assert!(check(root, &config("{trackedRoutesOnly: invalid}")).is_err());
    let files = fixture_files(root);
    let error = check_with_files(root, &config("{trackedRoutesOnly: true}"), &files).unwrap_err();
    assert!(error.to_string().contains("prepared Git index inventory"));
    let non_git = crate::test_support::materialize_saved_fixture(&fixture("tracked-routes"));
    let error = check(non_git.path(), &config("{trackedRoutesOnly: true}")).unwrap_err();
    assert!(error.to_string().contains("prepared Git index inventory"));
}

#[test]
fn failed_discovery_cannot_claim_a_git_index_inventory() {
    let root = fixture("pass");
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();
    let snapshot = VisiblePathSnapshot::new(&root);
    assert!(!snapshot.git_index_available_for(&root));
    assert!(snapshot.paths_for(&root).is_empty());
}
