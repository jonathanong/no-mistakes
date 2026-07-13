use super::*;

fn root_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-discovery")
            .join(name)
            .join("fixture"),
    )
}

fn assert_scoped_external_includes(views: &super::super::views::CheckFileViews) {
    for files in [&views.filesystem, &views.graph] {
        assert!(files
            .iter()
            .any(|path| path.ends_with("external-project/fixtures/included.ts")));
        assert!(files
            .iter()
            .any(|path| { path.ends_with("external-project/generated/explicitly-included.ts") }));
        assert!(!files
            .iter()
            .any(|path| path.ends_with("external-project/src/unrelated.ts")));
        assert!(files
            .iter()
            .any(|path| path.ends_with("typed-app/fixtures/included.ts")));
    }
}

#[test]
fn git_and_known_no_git_roots_preserve_external_project_includes() {
    let root = root_fixture("external-project-include");
    let config = load_config(&root);

    let git_views =
        discover_check_file_views(&root, &config, &config.filesystem.skip_directories, false);
    assert_scoped_external_includes(&git_views);

    let fallback_views = super::super::views::discover_check_file_views_from_git_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );
    assert_scoped_external_includes(&fallback_views);
}

#[test]
fn mixed_git_availability_walks_external_base_with_its_patterns_once() {
    let root = root_fixture("external-project-include");
    let config = load_config(&root);
    let root_files = no_mistakes::codebase::ts_source::git_visible_files(&root)
        .expect("workspace fixture should have a git-visible root universe");
    let external_root = root
        .parent()
        .expect("fixture root should have a parent")
        .join("external-project");
    let mut lookups = Vec::new();

    let views = super::super::views::discover_check_file_views_with_external_lookup(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        Some(root_files),
        |base| {
            lookups.push(base.to_path_buf());
            None
        },
    );

    assert_eq!(lookups, vec![external_root]);
    assert_scoped_external_includes(&views);
}

#[test]
fn known_no_git_reopens_forbidden_workspace_project_under_skipped_root() {
    let root = fixture("rules/filesystem-dispatch/forbidden-workspace-project-root");
    let config = load_config(&root);
    let views = super::super::views::discover_check_file_views_from_git_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );

    for files in [&views.filesystem, &views.graph] {
        assert!(files
            .iter()
            .any(|path| path.ends_with("fixtures/app/package.json")));
    }
}

#[test]
fn known_no_git_reopens_unique_exports_project_under_skipped_root() {
    let root = fixture("check-discovery/unique-exports-under-skipped-root");
    let config = load_config(&root);
    let views = super::super::views::discover_check_file_views_from_git_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        true,
        None,
    );

    for files in [&views.filesystem, &views.graph] {
        assert!(files
            .iter()
            .any(|path| path.ends_with("fixtures/app/src/index.ts")));
    }
}

#[test]
fn typed_project_root_resolution_uses_only_precomputed_file_universe() {
    let root = PathBuf::from("/virtual/repository");
    let expected = root.join("web");
    let files = vec![expected.join("next.config.ts")];
    let mut inferred = super::super::views::infer_project_roots_from_files(&root, &files);
    let project = no_mistakes::config::v2::schema::Project {
        type_: Some(no_mistakes::config::v2::schema::ProjectType::Nextjs),
        ..Default::default()
    };

    assert_eq!(project_root(&root, &project, &mut inferred), Some(expected));
}

#[test]
fn fallback_universe_is_git_free_and_applies_ignore_aware_pruning() {
    let root = root_fixture("external-project-include");
    let config = load_config(&root);
    let project_root = root
        .parent()
        .expect("fixture root should have a parent")
        .join("external-project");
    let mut inferred = no_mistakes::codebase::config::InferredRoots {
        nextjs: Some(None),
        remix: Some(None),
        vitejs: Some(None),
    };
    let patterns = super::super::preserved_roots::include_patterns_by_base_with_inferred(
        &root,
        &config,
        &mut inferred,
    );
    let files = super::super::views::walk_ignore_aware_universe(
        &project_root,
        patterns
            .get(&project_root)
            .map(Vec::as_slice)
            .unwrap_or_default(),
        &[],
        &[],
    );

    assert!(files
        .iter()
        .any(|path| path.ends_with("fixtures/included.ts")));
    assert!(files
        .iter()
        .any(|path| path.ends_with("generated/explicitly-included.ts")));
    assert!(files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ci.yml")));
    for excluded in [
        ".hidden/trap.ts",
        ".github/other/trap.yml",
        "node_modules/package/trap.ts",
        "ignored-store/trap.ts",
    ] {
        assert!(!files.iter().any(|path| path.ends_with(excluded)));
    }
}

#[test]
fn known_no_git_reopens_project_patterns_under_skipped_ancestors() {
    let root = root_fixture("project-pattern-reopen");
    let config = load_config(&root);
    let views = super::super::views::discover_check_file_views_from_git_files(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        None,
    );

    for files in [&views.filesystem, &views.graph] {
        assert!(files
            .iter()
            .any(|path| path.ends_with("fixtures/build/components/included.ts")));
        assert!(files
            .iter()
            .any(|path| path.ends_with("web/fixtures/included.ts")));
        assert!(files
            .iter()
            .any(|path| path.ends_with("web/src/node_modules/foo/included.ts")));
    }
}
