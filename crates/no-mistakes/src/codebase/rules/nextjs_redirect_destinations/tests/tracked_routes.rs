use super::*;
use crate::codebase::ts_source::VisiblePathSnapshot;
mod scoped_inventory;

fn tracked_fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("tracked-routes"));
    prepare_tracked_fixture(fixture.path());
    fixture
}

fn prepare_tracked_fixture(root: &Path) {
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
    let message = error.to_string();
    for expected in [
        "prepared Git index inventory",
        "fails closed",
        "Git-backed configured project",
        "run_filesystem_rules_with_files()",
        "trackedRoutesOnly: false",
    ] {
        assert!(message.contains(expected), "{message}");
    }
    let non_git = crate::test_support::materialize_saved_fixture(&fixture("tracked-routes"));
    let error = check(non_git.path(), &config("{trackedRoutesOnly: true}")).unwrap_err();
    let message = error.to_string();
    for expected in [
        "prepared Git index inventory",
        "fails closed",
        "Git-backed configured project",
        "run_filesystem_rules_with_files()",
        "trackedRoutesOnly: false",
    ] {
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn failed_discovery_cannot_claim_a_git_index_inventory() {
    let root = fixture("pass");
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();
    let snapshot = VisiblePathSnapshot::new(&root);
    assert!(!snapshot.git_index_available_for(&root));
    assert!(snapshot.paths_for(&root).is_empty());
}

#[test]
fn public_supplied_tracked_list_preserves_its_authority_without_git() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("tracked-routes"));
    let root = fixture.path();
    // This API's caller guarantees index membership; no Git discovery is needed.
    let files = vec![
        root.join("next.config.ts"),
        root.join("app/tracked/page.tsx"),
    ];
    let findings = crate::codebase::rules::run_filesystem_rules_with_files(
        root,
        Some(&root.join(".no-mistakes.yml")),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 10, "{findings:?}");
    let ordinary = VisiblePathSnapshot::from_paths(root, &files);
    assert!(!ordinary.git_index_available_for(root));
    let authoritative = VisiblePathSnapshot::from_tracked_paths(root, &files);
    assert!(authoritative.git_index_available_for(root));
    assert_eq!(authoritative.tracked_paths_from(&files), files);
    assert!(authoritative.git_index_available_for(&root.join("app")));
    assert!(authoritative.git_index_available_for(&root.parent().unwrap().join("outside-project")));
    assert!(std::sync::Arc::ptr_eq(
        &authoritative.source_store_for(root),
        &authoritative.source_store_for(&root.parent().unwrap().join("outside-project")),
    ));
}

#[test]
fn nested_git_projects_use_their_index_not_the_umbrella_fallback() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("nested-projects"));
    let root = fixture.path();
    let project = root.join("packages/web");
    prepare_tracked_fixture(&project);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = VisiblePathSnapshot::new_observed(root, Some(observer.clone()));
    assert!(!snapshot.git_index_available_for(root));
    let files = snapshot.paths_for(root);
    assert!(files.contains(&project.join("app/untracked/page.tsx")));
    let expected = crate::codebase::rules::run_filesystem_rules_with_visible_and_snapshot(
        root,
        Some(&root.join(".no-mistakes.yml")),
        &files,
        &snapshot,
    )
    .unwrap();
    assert_eq!(expected.len(), 6, "{expected:?}");
    assert!(snapshot.git_index_available_for(&project));
    assert_eq!(observer.snapshot().work["discovery.roots"], 2);
    assert_eq!(
        crate::codebase::rules::run_filesystem_rules_with_visible_and_snapshot(
            root,
            Some(&root.join(".no-mistakes.yml")),
            &files,
            &snapshot,
        )
        .unwrap(),
        expected
    );
    assert_eq!(observer.snapshot().work["discovery.roots"], 2);
    assert_eq!(
        crate::codebase::rules::run_filesystem_rules(root, Some(&root.join(".no-mistakes.yml")),)
            .unwrap(),
        expected
    );
    let supplied = snapshot.tracked_paths_for(&project);
    assert_eq!(
        crate::codebase::rules::run_filesystem_rules_with_files(
            root,
            Some(&root.join(".no-mistakes.yml")),
            &supplied,
        )
        .unwrap(),
        expected
    );
    crate::test_support::git_add_force(&project, &["app/untracked", "app/ignored"]);
    assert!(crate::codebase::rules::run_filesystem_rules(
        root,
        Some(&root.join(".no-mistakes.yml")),
    )
    .unwrap()
    .is_empty());
    let (project_config, _) =
        crate::config::v2::load_v2_config_with_path(root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    assert!(check(root, &project_config).unwrap().is_empty());
    // A supplied list remains authoritative across a nested Git boundary.
    assert_eq!(
        crate::codebase::rules::run_filesystem_rules_with_files(
            root,
            Some(&root.join(".no-mistakes.yml")),
            &supplied,
        )
        .unwrap(),
        expected
    );
}

#[test]
fn nested_projects_without_a_prepared_index_still_fail_closed() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("nested-projects"));
    let root = fixture.path();
    let error =
        crate::codebase::rules::run_filesystem_rules(root, Some(&root.join(".no-mistakes.yml")))
            .unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared Git index inventory for"));
}

#[test]
fn authoritative_routes_still_honor_scope_skip_and_rule_path_filters() {
    let root = fixture("tracked-routes");
    let files = vec![
        root.join("next.config.ts"),
        root.join("app/tracked/page.tsx"),
        fixture("pass").join("app/about/page.tsx"),
    ];
    let snapshot = VisiblePathSnapshot::from_tracked_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let mut config = config("{trackedRoutesOnly: true}");
    config.filesystem.skip_directories = vec!["app".to_string()];
    assert_eq!(
        check_with_files_sources_and_snapshot(&root, &config, &files, &sources, Some(&snapshot),)
            .unwrap()
            .len(),
        11
    );
    config.filesystem.skip_directories.clear();
    config.rules[0].exclude = vec!["app/**".to_string()];
    assert_eq!(
        check_with_files_sources_and_snapshot(&root, &config, &files, &sources, Some(&snapshot),)
            .unwrap()
            .len(),
        11
    );
}

#[test]
fn overlapping_project_routes_cannot_borrow_another_projects_git_index() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("overlapping-projects"));
    let root = fixture.path();
    let outer = root.join("web");
    let embedded = outer.join("app/embedded");
    crate::test_support::git_init(&outer);
    crate::test_support::git_init(&embedded);
    crate::test_support::git_add_force(&outer, &["next.config.ts", "app/outer"]);
    crate::test_support::git_add_force(&embedded, &["next.config.ts", "app/present"]);
    let findings =
        crate::codebase::rules::run_filesystem_rules(root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "web/next.config.ts");
    assert!(findings[0].message.contains("'/embedded/app/present'"));
}
