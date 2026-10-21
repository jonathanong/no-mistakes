use super::*;
use crate::codebase::workspaces::preparation_work;

fn fixture() -> (PathBuf, Vec<PathBuf>, std::sync::Arc<SourceStore>) {
    let root = fixture_root("prepared-workspace");
    let files = [
        "package.json",
        "packages/a/package.json",
        "packages/a/src/main.mts",
        "packages/a/module.mts",
        "packages/a/main.mts",
        "packages/a/nested/package.json",
        "packages/a/nested/types.d.ts",
        "packages/a/nested/src/deep.mts",
        "packages/consumer/package.json",
        "packages/consumer/src/index.mts",
        "packages/z/package.json",
        "packages/z/module.mts",
        "outside/package.json",
        "outside/index.mts",
        "README.md",
    ]
    .map(|path| root.join(path))
    .to_vec();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    (root, files, sources)
}

#[test]
fn visibility_is_prepared_once_across_packages_and_overlapping_roots() {
    let (root, files, sources) = fixture();
    preparation_work::reset();
    let roots = [
        root.join("packages"),
        root.join("packages/a"),
        root.join("packages"),
    ];
    let workspace = load_workspace(&roots, &files, &sources).unwrap();
    assert_eq!(preparation_work::counts().visible_paths, files.len());
    assert_eq!(workspace.packages.len(), 3);
    let duplicate = workspace.package_by_name("@fixture/duplicate").unwrap();
    assert_eq!(duplicate.dir, root.join("packages/a"));
    assert_eq!(duplicate.entry, Some(root.join("packages/a/src/main.mts")));
    assert!(workspace.package_by_name("fixture-root").is_none());
    assert!(workspace.package_by_name("@fixture/outside").is_none());
    let nested = workspace.package_by_name("@fixture/nested").unwrap();
    assert_eq!(
        nested.entry,
        Some(root.join("packages/a/nested/types.d.ts"))
    );

    // Distinct overlapping roots keep their historical iteration order.
    let reversed = load_workspace(&[roots[1].clone(), roots[0].clone()], &files, &sources).unwrap();
    let duplicate = reversed.package_by_name("@fixture/duplicate").unwrap();
    assert_eq!(duplicate.dir, root.join("packages/z"));
    assert_eq!(duplicate.entry, Some(root.join("packages/z/module.mts")));
}

#[test]
fn ownership_prepares_each_directory_once_and_probes_only_ancestors() {
    let (root, mut files, sources) = fixture();
    let workspace = load_workspace(&[root.join("packages")], &files, &sources).unwrap();
    files.push(root.join("packages/a/nested/src/../src/deep.mts"));
    preparation_work::reset();
    let owners = compute_owners(&workspace, &files);
    let counts = preparation_work::counts();
    assert_eq!(counts.package_dirs, workspace.packages.len());
    let expected_lookups: usize = files
        .iter()
        .map(|file| {
            let file = normalize_path(file);
            file.ancestors()
                .take_while(|dir| !workspace.packages.iter().any(|p| p.dir == *dir))
                .count()
                + usize::from(owners.contains_key(&file))
        })
        .sum();
    assert_eq!(counts.ancestor_lookups, expected_lookups);
    assert_eq!(
        owners.get(&root.join("packages/a/nested/src/deep.mts")),
        Some(&root.join("packages/a/nested"))
    );
    assert!(!owners.contains_key(&root.join("outside/index.mts")));
    // The duplicate-name loser is not retained as a package owner.
    assert!(!owners.contains_key(&root.join("packages/a/src/main.mts")));
}

#[test]
fn empty_selection_never_prepares_full_visibility() {
    let (root, files, sources) = fixture();
    preparation_work::reset();
    assert!(
        load_workspace(&[root.join("not-selected")], &files, &sources)
            .unwrap()
            .packages
            .is_empty()
    );
    assert_eq!(preparation_work::counts().visible_paths, 0);
}

#[test]
fn saved_workspace_keeps_exact_dependency_findings() {
    let (root, files, _) = fixture();
    let config: crate::config::v2::NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let findings = crate::codebase::rules::production_dependency_declarations::check_with_files(
        &root, &config, &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "packages/z/module.mts");
    assert_eq!(findings[0].line, 1);
    assert_eq!(
        findings[0].import.as_deref(),
        Some("missing-runtime-dependency")
    );
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-runtime-dependency")
    );
}

#[test]
fn prepared_visibility_keeps_the_application_scope_and_normalizes_members() {
    let (root, _, sources) = fixture();
    // The SourceStore also contains the higher-priority export and other packages.
    // Only this application's explicit paths may answer entry resolution.
    let files = vec![
        root.join("packages/a/package.json"),
        root.join("packages/a/src/../module.mts"),
    ];
    let visible = std::sync::OnceLock::new();
    preparation_work::reset();
    let workspace =
        super::super::load_workspace(&[root.join("packages/a")], &files, &sources, &visible)
            .unwrap();
    let package = workspace.package_by_name("@fixture/duplicate").unwrap();
    assert_eq!(package.entry, Some(root.join("packages/a/module.mts")));
    let visible = visible.get().unwrap();
    assert_eq!(visible.len(), files.len());
    assert!(visible.contains(&root.join("packages/a/module.mts")));
    assert!(!visible.contains(&root.join("packages/a/src/main.mts")));
    assert_eq!(preparation_work::counts().visible_paths, files.len());
}
