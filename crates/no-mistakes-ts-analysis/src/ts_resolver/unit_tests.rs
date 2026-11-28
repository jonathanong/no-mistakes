use super::*;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn import_resolution_cache_clear_removes_raw_and_classified_entries() {
    let root = workspace_tsconfig_fixture();
    let importer = root.join("apps/web/src/entry.ts");
    let target = root.join("apps/web/src/runtime/value.ts");
    let cache = ImportResolutionCache::default();
    let key = ResolveKey {
        importing_file: Arc::from(importer),
        specifier: Arc::from("@runtime/value"),
    };
    cache.raw_entries.insert(key.clone(), Some(target.clone()));
    let classification_key = ClassificationKey {
        importing_file: Arc::clone(&key.importing_file),
        specifier: Arc::clone(&key.specifier),
        policy: ClassificationPolicy::Standard,
    };
    cache.final_entries.insert(
        classification_key.clone(),
        ImportClassification {
            resolver_target: Some(target),
            workspace_target: None,
            workspace_recognized: false,
        },
    );

    cache.clear();

    assert!(cache.raw_entries.get(&key).is_none());
    assert!(cache.final_entries.get(&classification_key).is_none());
}

#[test]
fn interned_resolve_keys_hit_shared_cache() {
    let interner = crate::path_interner::PathInterner::new();
    let first = ResolveKey {
        importing_file: interner.intern_path("src/a.ts"),
        specifier: interner.intern_str("./b"),
    };
    let second = ResolveKey {
        importing_file: interner.intern_path("src/./a.ts"),
        specifier: interner.intern_str("./b"),
    };
    assert!(Arc::ptr_eq(&first.importing_file, &second.importing_file));
    assert_eq!(first, second);

    let cache = ImportResolutionCache::default();
    cache.raw_entries.insert(first.clone(), None);
    assert!(cache.raw_entries.get(&second).is_some());
}

#[test]
fn local_arc_resolve_keys_match_interned_keys_by_path_bytes() {
    let interner = crate::path_interner::PathInterner::new();
    let interned = ResolveKey {
        importing_file: interner.intern_path("src/./a.ts"),
        specifier: interner.intern_str("./b"),
    };
    let local = ResolveKey {
        importing_file: Arc::from(normalize_path(Path::new("src/a.ts"))),
        specifier: Arc::from("./b"),
    };
    assert!(!Arc::ptr_eq(
        &interned.importing_file,
        &local.importing_file
    ));
    assert_eq!(interned, local);

    let cache = ImportResolutionCache::default();
    cache.raw_entries.insert(interned, None);
    assert!(cache.raw_entries.get(&local).is_some());
}

fn saved_resolver_fixture(name: &str) -> TempDir {
    let source = crate::test_support::fixture_root("ts-resolver").join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/ts-resolver/fixture")
        .join(name)
}

fn workspace_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

fn absolute_glob_rules_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/absolute-glob-rules"),
    )
}

fn exact_include_files_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/exact-include-files"),
    )
}

fn include_config_directory_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/include-config-directory"),
    )
}

fn trailing_directory_include_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/trailing-directory-include"),
    )
}

fn paths_precedence_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/paths-precedence"),
    )
}

#[path = "tests/aliases.rs"]
mod aliases;
#[path = "tests/catalog_config.rs"]
mod catalog_config;
#[path = "tests/config_loading_and_relative.rs"]
mod config_loading_and_relative;
#[path = "tests/extends.rs"]
mod extends;
