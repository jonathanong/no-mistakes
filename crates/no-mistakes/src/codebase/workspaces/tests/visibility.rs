use super::super::*;

#[test]
fn ignored_pnpm_manifest_does_not_override_visible_package_workspaces() {
    let fixture = crate::test_support::materialize_gitignore_fixture("workspace-ignored-pnpm");

    let workspace = load(fixture.path()).unwrap();

    assert_eq!(workspace.packages.len(), 1);
    assert_eq!(workspace.packages[0].name, "@fixture/visible-npm");
}

#[test]
fn ignored_root_package_is_not_automatic_workspace_metadata() {
    let fixture =
        crate::test_support::materialize_gitignore_fixture("workspace-ignored-root-package");
    let files = crate::codebase::ts_source::discover_visible_paths(fixture.path());

    assert!(load_from_files(fixture.path(), &files)
        .unwrap()
        .packages
        .is_empty());
    assert!(load_root_package_from_files(fixture.path(), &files)
        .unwrap()
        .is_none());
}

#[test]
fn tracked_ignored_root_package_remains_workspace_metadata() {
    let fixture =
        crate::test_support::materialize_gitignore_fixture("workspace-ignored-root-package");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    crate::test_support::git_add_force(fixture.path(), &["package.json"]);
    let files = crate::codebase::ts_source::discover_visible_paths(fixture.path());

    let workspace = load_from_files(fixture.path(), &files).unwrap();
    let root_package = load_root_package_from_files(fixture.path(), &files)
        .unwrap()
        .expect("tracked root package remains visible");

    assert_eq!(workspace.packages.len(), 1);
    assert_eq!(workspace.packages[0].name, "@fixture/app");
    assert_eq!(root_package.name, "@fixture/root");
}

#[test]
fn pass4a_ignored_workspace_candidates_do_not_shadow_visible_entry_and_subpath_fallbacks() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4a-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = normalize_path(fixture.path());
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let visible_files = files.iter().cloned().collect();

    let workspace = load_from_files(&root, &files).unwrap();
    let indexed_workspace = load_indexed_from_files(&root, &files).unwrap();
    let package = workspace
        .packages
        .iter()
        .find(|package| package.name == "@fixture/pkg")
        .unwrap();

    assert_eq!(package.entry, Some(root.join("packages/pkg/src/main.ts")));
    assert_eq!(
        workspace.resolve_specifier_from_visible("@fixture/pkg/feature", &visible_files),
        Some(root.join("packages/pkg/src/feature.ts"))
    );
    let importing_file = root.join("packages/pkg/src/main.ts");
    assert!(workspace.recognizes_specifier_from("@fixture/pkg", &importing_file));
    assert!(workspace.recognizes_specifier_from("@fixture/pkg/ignored", &importing_file));
    assert!(workspace.recognizes_specifier_from("#module", &importing_file));
    assert!(!workspace.recognizes_specifier_from("#missing", &importing_file));
    assert!(!workspace.recognizes_specifier_from("external-package", &importing_file));
    assert!(indexed_workspace.recognizes_specifier_from("#module", &importing_file));
    assert!(!indexed_workspace.recognizes_specifier_from("#missing", &importing_file));
}
