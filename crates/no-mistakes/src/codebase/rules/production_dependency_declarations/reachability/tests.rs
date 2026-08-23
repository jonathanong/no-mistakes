use super::*;
use crate::codebase::workspaces::WorkspacePackage;

fn fixture_root(name: &str) -> PathBuf {
    normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-dependency-declarations")
            .join(name),
    )
}

fn globset(patterns: &[&str]) -> GlobSet {
    let mut builder = globset::GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(globset::Glob::new(pattern).unwrap());
    }
    builder.build().unwrap()
}

#[test]
fn file_imports_extracts_specifier_kind_and_line() {
    let root = fixture_root("dev-only-production-import");
    let file = root.join("packages/lib/index.mts");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));

    let imports = file_imports(&file, &sources);

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].specifier, "@acme/tool");
    assert_eq!(imports[0].kind, ImportKind::Static);
    assert_eq!(imports[0].line, 1);
}

#[test]
fn file_imports_returns_empty_for_a_file_absent_from_sources() {
    let sources = crate::codebase::rules::source_store_for_files(&[]);

    let imports = file_imports(Path::new("/does/not/exist.mts"), &sources);

    assert!(imports.is_empty());
}

#[test]
fn file_imports_uses_the_tsx_extractor_for_a_tsx_file() {
    let root = fixture_root("dev-only-production-import");
    let file = root.join("packages/lib/widget.tsx");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));

    let imports = file_imports(&file, &sources);

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].specifier, "@acme/tool");
    assert_eq!(imports[0].kind, ImportKind::Static);
}

#[test]
fn try_resolve_matches_an_exact_visible_path() {
    let candidate = PathBuf::from("/repo/packages/lib/helper.mts");
    let visible: crate::fx::PathSet = [candidate.clone()].into_iter().collect();

    assert_eq!(try_resolve(&candidate, &visible), Some(candidate));
}

#[test]
fn try_resolve_appends_a_resolve_extension() {
    let candidate = PathBuf::from("/repo/packages/lib/helper");
    let resolved = PathBuf::from("/repo/packages/lib/helper.mts");
    let visible: crate::fx::PathSet = [resolved.clone()].into_iter().collect();

    assert_eq!(try_resolve(&candidate, &visible), Some(resolved));
}

#[test]
fn try_resolve_falls_back_to_a_directory_index_file() {
    let candidate = PathBuf::from("/repo/packages/lib/sub");
    let resolved = PathBuf::from("/repo/packages/lib/sub/index.mts");
    let visible: crate::fx::PathSet = [resolved.clone()].into_iter().collect();

    assert_eq!(try_resolve(&candidate, &visible), Some(resolved));
}

#[test]
fn try_resolve_returns_none_when_nothing_matches() {
    let candidate = PathBuf::from("/repo/packages/lib/missing");

    assert_eq!(
        try_resolve(&candidate, &crate::fx::PathSet::default()),
        None
    );
}

#[test]
fn resolve_relative_joins_the_importing_files_directory() {
    let file = PathBuf::from("/repo/packages/lib/index.mts");
    let resolved = PathBuf::from("/repo/packages/lib/helper.mts");
    let visible: crate::fx::PathSet = [resolved.clone()].into_iter().collect();

    assert_eq!(
        resolve_relative(&file, "./helper", &visible),
        Some(resolved)
    );
}

#[test]
fn resolve_relative_returns_none_for_a_file_with_no_parent() {
    let file = PathBuf::from("");

    assert_eq!(
        resolve_relative(&file, "./helper", &crate::fx::PathSet::default()),
        None
    );
}

#[test]
fn resolve_target_dispatches_relative_specifiers_to_resolve_relative() {
    let workspace = WorkspaceMap::from_packages(Vec::new());
    let file = PathBuf::from("/repo/packages/lib/index.mts");
    let resolved = PathBuf::from("/repo/packages/lib/helper.mts");
    let visible: crate::fx::PathSet = [resolved.clone()].into_iter().collect();

    assert_eq!(
        resolve_target(&workspace, &file, "./helper", &visible),
        Some(resolved)
    );
}

#[test]
fn resolve_target_dispatches_bare_specifiers_to_the_workspace() {
    let entry = PathBuf::from("/repo/packages/tool/index.mts");
    let workspace = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/tool".to_string(),
        dir: PathBuf::from("/repo/packages/tool"),
        entry: Some(entry.clone()),
        exports: None,
        imports: None,
    }]);
    let file = PathBuf::from("/repo/packages/lib/index.mts");
    let visible: crate::fx::PathSet = [entry.clone()].into_iter().collect();

    assert_eq!(
        resolve_target(&workspace, &file, "@acme/tool", &visible),
        Some(entry)
    );
}

#[test]
fn resolved_targets_filters_out_type_only_imports() {
    let entry = PathBuf::from("/repo/packages/tool/index.mts");
    let workspace = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/tool".to_string(),
        dir: PathBuf::from("/repo/packages/tool"),
        entry: Some(entry.clone()),
        exports: None,
        imports: None,
    }]);
    let file = PathBuf::from("/repo/packages/lib/index.mts");
    let visible: crate::fx::PathSet = [entry.clone()].into_iter().collect();
    let imports = vec![
        FileImport {
            line: 1,
            specifier: "@acme/tool".to_string(),
            kind: ImportKind::Type,
        },
        FileImport {
            line: 2,
            specifier: "@acme/tool".to_string(),
            kind: ImportKind::Static,
        },
    ];

    assert_eq!(
        resolved_targets(&workspace, &file, &imports, &visible),
        vec![entry]
    );
}

#[test]
fn production_reachable_files_seeds_from_an_external_importer_and_follows_relative_imports() {
    let root = PathBuf::from("/repo");
    let package_dir = PathBuf::from("/repo/packages/lib");
    let lib_entry = PathBuf::from("/repo/packages/lib/index.mts");
    let lib_helper = PathBuf::from("/repo/packages/lib/helper.mts");
    let app_dir = PathBuf::from("/repo/packages/app");
    let app_entry = PathBuf::from("/repo/packages/app/index.mts");

    let workspace = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/lib".to_string(),
        dir: package_dir.clone(),
        entry: Some(lib_entry.clone()),
        exports: None,
        imports: None,
    }]);
    let package_files: HashSet<PathBuf> = [lib_entry.clone(), lib_helper.clone()]
        .into_iter()
        .collect();
    let mut imports_by_file = HashMap::new();
    imports_by_file.insert(
        app_entry.clone(),
        vec![FileImport {
            line: 1,
            specifier: "@acme/lib".to_string(),
            kind: ImportKind::Static,
        }],
    );
    imports_by_file.insert(
        lib_entry.clone(),
        vec![FileImport {
            line: 1,
            specifier: "./helper".to_string(),
            kind: ImportKind::Static,
        }],
    );
    imports_by_file.insert(lib_helper.clone(), Vec::new());
    let mut owners = HashMap::new();
    owners.insert(app_entry.clone(), app_dir);
    owners.insert(lib_entry.clone(), package_dir.clone());
    owners.insert(lib_helper.clone(), package_dir.clone());
    let visible: crate::fx::PathSet = [app_entry.clone(), lib_entry.clone(), lib_helper.clone()]
        .into_iter()
        .collect();
    let test_globset = globset(&["**/__tests__/**"]);
    let ctx = ReachabilityContext {
        root: &root,
        workspace: &workspace,
        imports_by_file: &imports_by_file,
        owners: &owners,
        test_globset: &test_globset,
        visible: &visible,
    };

    let reachable = production_reachable_files(&ctx, &package_dir, &package_files);

    assert_eq!(
        reachable,
        [lib_entry, lib_helper].into_iter().collect::<HashSet<_>>()
    );
}

#[test]
fn production_reachable_files_excludes_test_only_importers_from_seeding() {
    let root = PathBuf::from("/repo");
    let package_dir = PathBuf::from("/repo/packages/lib");
    let lib_entry = PathBuf::from("/repo/packages/lib/index.mts");
    let app_dir = PathBuf::from("/repo/packages/app");
    let test_file = PathBuf::from("/repo/packages/app/__tests__/index.test.mts");

    let workspace = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/lib".to_string(),
        dir: package_dir.clone(),
        entry: Some(lib_entry.clone()),
        exports: None,
        imports: None,
    }]);
    let package_files: HashSet<PathBuf> = [lib_entry.clone()].into_iter().collect();
    let mut imports_by_file = HashMap::new();
    imports_by_file.insert(
        test_file.clone(),
        vec![FileImport {
            line: 1,
            specifier: "@acme/lib".to_string(),
            kind: ImportKind::Static,
        }],
    );
    let mut owners = HashMap::new();
    owners.insert(test_file.clone(), app_dir);
    let visible: crate::fx::PathSet = [lib_entry, test_file].into_iter().collect();
    let test_globset = globset(&["**/__tests__/**"]);
    let ctx = ReachabilityContext {
        root: &root,
        workspace: &workspace,
        imports_by_file: &imports_by_file,
        owners: &owners,
        test_globset: &test_globset,
        visible: &visible,
    };

    let reachable = production_reachable_files(&ctx, &package_dir, &package_files);

    assert!(reachable.is_empty());
}

#[test]
fn production_reachable_files_treats_a_missing_imports_entry_as_a_leaf() {
    let root = PathBuf::from("/repo");
    let package_dir = PathBuf::from("/repo/packages/lib");
    let lib_entry = PathBuf::from("/repo/packages/lib/index.mts");
    let app_dir = PathBuf::from("/repo/packages/app");
    let app_entry = PathBuf::from("/repo/packages/app/index.mts");

    let workspace = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/lib".to_string(),
        dir: package_dir.clone(),
        entry: Some(lib_entry.clone()),
        exports: None,
        imports: None,
    }]);
    let package_files: HashSet<PathBuf> = [lib_entry.clone()].into_iter().collect();
    let mut imports_by_file = HashMap::new();
    imports_by_file.insert(
        app_entry.clone(),
        vec![FileImport {
            line: 1,
            specifier: "@acme/lib".to_string(),
            kind: ImportKind::Static,
        }],
    );
    // Deliberately no entry for `lib_entry`: the BFS must still terminate
    // gracefully when a reachable file was never fed through `file_imports`.
    let mut owners = HashMap::new();
    owners.insert(app_entry.clone(), app_dir);
    let visible: crate::fx::PathSet = [app_entry, lib_entry.clone()].into_iter().collect();
    let test_globset = globset(&["**/__tests__/**"]);
    let ctx = ReachabilityContext {
        root: &root,
        workspace: &workspace,
        imports_by_file: &imports_by_file,
        owners: &owners,
        test_globset: &test_globset,
        visible: &visible,
    };

    let reachable = production_reachable_files(&ctx, &package_dir, &package_files);

    assert_eq!(reachable, [lib_entry].into_iter().collect::<HashSet<_>>());
}

#[test]
fn production_reachable_files_does_not_seed_from_files_owned_by_the_same_package() {
    let root = PathBuf::from("/repo");
    let package_dir = PathBuf::from("/repo/packages/lib");
    let lib_entry = PathBuf::from("/repo/packages/lib/index.mts");
    let lib_helper = PathBuf::from("/repo/packages/lib/helper.mts");

    let workspace = WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/lib".to_string(),
        dir: package_dir.clone(),
        entry: Some(lib_entry.clone()),
        exports: None,
        imports: None,
    }]);
    let package_files: HashSet<PathBuf> = [lib_entry.clone(), lib_helper.clone()]
        .into_iter()
        .collect();
    let mut imports_by_file = HashMap::new();
    imports_by_file.insert(
        lib_helper.clone(),
        vec![FileImport {
            line: 1,
            specifier: "@acme/lib".to_string(),
            kind: ImportKind::Static,
        }],
    );
    let mut owners = HashMap::new();
    owners.insert(lib_helper.clone(), package_dir.clone());
    let visible: crate::fx::PathSet = [lib_entry, lib_helper].into_iter().collect();
    let test_globset = globset(&["**/__tests__/**"]);
    let ctx = ReachabilityContext {
        root: &root,
        workspace: &workspace,
        imports_by_file: &imports_by_file,
        owners: &owners,
        test_globset: &test_globset,
        visible: &visible,
    };

    let reachable = production_reachable_files(&ctx, &package_dir, &package_files);

    assert!(reachable.is_empty());
}
