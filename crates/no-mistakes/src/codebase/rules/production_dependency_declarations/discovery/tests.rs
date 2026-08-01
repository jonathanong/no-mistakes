use super::*;

fn fixture_root(name: &str) -> PathBuf {
    normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-dependency-declarations")
            .join(name),
    )
}

#[test]
fn load_workspace_discovers_nothing_without_configured_workspace_roots() {
    let root = fixture_root("dev-only-production-import");
    let files = vec![
        root.join("packages/app/package.json"),
        root.join("packages/app/index.mts"),
        root.join("packages/lib/package.json"),
        root.join("packages/lib/index.mts"),
        root.join("packages/tool/package.json"),
        root.join("packages/tool/index.mts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);

    let workspace = load_workspace(&[], &files, &sources).unwrap();

    assert_eq!(workspace.packages.len(), 0);
}

#[test]
fn load_workspace_restricts_discovery_to_configured_workspace_roots() {
    let root = fixture_root("dev-only-production-import");
    let files = vec![
        root.join("packages/app/package.json"),
        root.join("packages/app/index.mts"),
        root.join("packages/lib/package.json"),
        root.join("packages/lib/index.mts"),
        root.join("packages/tool/package.json"),
        root.join("packages/tool/index.mts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let app_root = root.join("packages/app");

    let workspace = load_workspace(&[app_root], &files, &sources).unwrap();

    assert_eq!(workspace.packages.len(), 1);
    assert!(workspace.package_by_name("@acme/app").is_some());
    assert!(workspace.package_by_name("@acme/lib").is_none());
    assert!(workspace.package_by_name("@acme/tool").is_none());
}

#[test]
fn load_workspace_dedupes_a_workspace_root_listed_twice() {
    let root = fixture_root("dependencies-declared");
    let files = vec![
        root.join("packages/app/package.json"),
        root.join("packages/app/index.mts"),
        root.join("packages/lib/package.json"),
        root.join("packages/lib/index.mts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);

    let workspace = load_workspace(&[root.clone(), root.clone()], &files, &sources).unwrap();

    assert_eq!(workspace.packages.len(), 2);
}

#[test]
fn load_workspace_unions_multiple_distinct_workspace_roots() {
    let root = fixture_root("dev-only-production-import");
    let files = vec![
        root.join("packages/app/package.json"),
        root.join("packages/app/index.mts"),
        root.join("packages/lib/package.json"),
        root.join("packages/lib/index.mts"),
        root.join("packages/tool/package.json"),
        root.join("packages/tool/index.mts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let app_root = root.join("packages/app");
    let lib_root = root.join("packages/lib");

    let workspace = load_workspace(&[app_root, lib_root], &files, &sources).unwrap();

    assert_eq!(workspace.packages.len(), 2);
    assert!(workspace.package_by_name("@acme/app").is_some());
    assert!(workspace.package_by_name("@acme/lib").is_some());
    assert!(workspace.package_by_name("@acme/tool").is_none());
}

#[test]
fn load_workspace_skips_a_manifest_without_a_usable_name_field() {
    let root = fixture_root("manifest-without-name");
    let files = vec![root.join("package.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);

    let workspace = load_workspace(&[root], &files, &sources).unwrap();

    assert_eq!(workspace.packages.len(), 0);
}

#[test]
fn load_workspace_ignores_non_manifest_files() {
    let root = fixture_root("undeclared-import");
    let files = vec![
        root.join("packages/app/package.json"),
        root.join("packages/app/index.mts"),
        root.join("packages/lib/package.json"),
        root.join("packages/lib/index.mts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);

    let workspace = load_workspace(&[root], &files, &sources).unwrap();

    assert!(workspace.package_by_name("left-pad").is_none());
}

#[test]
fn compute_owners_maps_each_file_to_its_nearest_package_directory() {
    let root = fixture_root("dev-only-production-import");
    let files = vec![
        root.join("packages/app/package.json"),
        root.join("packages/app/index.mts"),
        root.join("packages/lib/package.json"),
        root.join("packages/lib/index.mts"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let workspace = load_workspace(std::slice::from_ref(&root), &files, &sources).unwrap();

    let owners = compute_owners(&workspace, &files);

    let app_index = normalize_path(&root.join("packages/app/index.mts"));
    let lib_index = normalize_path(&root.join("packages/lib/index.mts"));
    assert_eq!(
        owners.get(&app_index),
        Some(&normalize_path(&root.join("packages/app")))
    );
    assert_eq!(
        owners.get(&lib_index),
        Some(&normalize_path(&root.join("packages/lib")))
    );
}

#[test]
fn compute_owners_excludes_files_outside_any_known_package() {
    let root = fixture_root("dev-only-production-import");
    let outside = normalize_path(&root.join("README.md"));
    let files = vec![root.join("packages/app/package.json"), outside.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let workspace = load_workspace(&[root], &files, &sources).unwrap();

    let owners = compute_owners(&workspace, &files);

    assert!(!owners.contains_key(&outside));
}

#[test]
fn group_by_package_inverts_owners_by_directory() {
    let root = fixture_root("dev-only-production-import");
    let app_dir = normalize_path(&root.join("packages/app"));
    let app_index = normalize_path(&root.join("packages/app/index.mts"));
    let app_package_json = normalize_path(&root.join("packages/app/package.json"));
    let mut owners = HashMap::new();
    owners.insert(app_index.clone(), app_dir.clone());
    owners.insert(app_package_json.clone(), app_dir.clone());

    let grouped = group_by_package(&owners);

    let files = grouped.get(&app_dir).unwrap();
    assert!(files.contains(&app_index));
    assert!(files.contains(&app_package_json));
}
