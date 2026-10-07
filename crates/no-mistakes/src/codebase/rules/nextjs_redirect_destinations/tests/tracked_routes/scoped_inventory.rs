use super::*;

#[test]
fn staged_ignored_nested_configs_share_the_request_source_read_identity() {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture("nested-ignored-config"));
    let root = fixture.path();
    let project = root.join("packages/web");
    prepare_tracked_fixture(&project);
    let snapshot = VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config_path = project.join("next.config.ts");
    assert!(!files.contains(&config_path));
    assert!(snapshot.paths_for(&project).contains(&config_path));
    let sources = snapshot.source_store_for(root);
    let (config, _) =
        crate::config::v2::load_v2_config_with_path(root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    let expected =
        check_with_files_sources_and_snapshot(root, &config, &files, &sources, Some(&snapshot))
            .unwrap();
    assert_eq!(expected.len(), 6, "{expected:?}");
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(
        check_with_files_sources_and_snapshot(root, &config, &files, &sources, Some(&snapshot))
            .unwrap(),
        expected
    );
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(check(root, &config).unwrap(), expected);
    assert_eq!(
        crate::codebase::rules::run_filesystem_rules(root, Some(&root.join(".no-mistakes.yml")))
            .unwrap(),
        expected
    );
    assert_eq!(
        crate::codebase::rules::run_filesystem_rules_with_visible_and_snapshot(
            root,
            Some(&root.join(".no-mistakes.yml")),
            &files,
            &snapshot
        )
        .unwrap(),
        expected
    );
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn sparse_and_locally_missing_pages_preserve_complete_index_membership() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    crate::test_support::git_add_force(root, &["app/untracked", "app/ignored"]);
    let sparse = root.join("app/tracked/page.tsx");
    let missing = root.join("app/untracked/page.tsx");
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["update-index", "--skip-worktree", "app/tracked/page.tsx"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(output.status.success());
    std::fs::remove_file(&sparse).unwrap();
    std::fs::remove_file(&missing).unwrap();
    let snapshot = VisiblePathSnapshot::new(root);
    assert!(!snapshot.paths_for(root).contains(&sparse));
    assert!(!snapshot.tracked_paths_for(root).contains(&missing));
    assert!(snapshot.git_index_paths_for(root).contains(&sparse));
    assert!(snapshot.git_index_paths_for(root).contains(&missing));
    let config = config("{trackedRoutesOnly: true}");
    assert!(check(root, &config).unwrap().is_empty());
    let supplied = snapshot.git_index_paths_for(root);
    assert!(crate::codebase::rules::run_filesystem_rules_with_files(
        root,
        Some(&root.join(".no-mistakes.yml")),
        &supplied
    )
    .unwrap()
    .is_empty());
    // Removing from the index changes membership even though a prepared view
    // remains frozen; sparse metadata alone never makes an unindexed page valid.
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["update-index", "--force-remove", "app/tracked/page.tsx"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(snapshot.git_index_paths_for(root).contains(&sparse));
    assert_eq!(check(root, &config).unwrap().len(), 1);
}
