use super::*;
use std::sync::Arc;
use tempfile::TempDir;

impl ImportResolutionCache {
    pub(crate) fn classification_count(&self) -> usize {
        self.classifications
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn request_count(&self) -> usize {
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }
}

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
fn normalized_visible_tsconfig_lookup_matches_visible_path_scan() {
    let root = workspace_tsconfig_fixture();
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let normalized_visible = visible_paths
        .iter()
        .map(|path| normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let cases = [
        root.join("apps/web/src/entry.ts"),
        root.join("packages/base-owner/src/value.ts"),
        root.join("missing/no-config.ts"),
    ];
    for source in cases {
        assert_eq!(
            find_tsconfig_from_normalized_visible(&source, &normalized_visible),
            find_tsconfig_from_visible(&source, &visible_paths),
            "normalized membership must preserve nearest-config selection for {}",
            source.display()
        );
    }

    let without_configs = visible_paths
        .into_iter()
        .filter(|path| path.file_name().is_none_or(|name| name != "tsconfig.json"))
        .collect::<Vec<_>>();
    let normalized_without_configs = without_configs
        .iter()
        .map(|path| normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let source = root.join("apps/web/src/entry.ts");
    assert_eq!(
        find_tsconfig_from_normalized_visible(&source, &normalized_without_configs),
        find_tsconfig_from_visible(&source, &without_configs),
        "an on-disk but omitted config must remain ignored"
    );
}

#[test]
fn interned_resolve_keys_hit_shared_cache() {
    let interner = crate::codebase::analysis_session::PathInterner::new();
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
fn scoped_shared_cache_attaches_session_interner() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let target = web.join("src/runtime/value.ts");
    let visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        importer.clone(),
        target.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web], &visible);
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let cache = ImportResolutionCache::default();
    let resolver = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_shared_cache(&cache);
    assert_eq!(session.interner().interned_str_count(), 0);
    assert_eq!(resolver.resolve("@runtime/value", &importer), Some(target));
    assert!(
        session.interner().interned_str_count() > 0,
        "shared-cache scoped resolvers must intern specifiers through the session"
    );
}

#[test]
fn local_arc_resolve_keys_match_interned_keys_by_path_bytes() {
    let interner = crate::codebase::analysis_session::PathInterner::new();
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

// Child modules still call write(); copy saved bytes instead of creating them inline.
fn write(path: &Path, content: &str) {
    ensure_parent(path);
    std::fs::copy(byte_blob(content), path).unwrap();
}

fn ensure_parent(path: &Path) {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return;
    };
    if parent.exists() {
        return;
    }
    ensure_parent(parent);
    std::fs::create_dir(parent).unwrap();
}

fn byte_blob(content: &str) -> PathBuf {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase/ts-resolver-byte-blobs")
        .join(format!("{hash:016x}"));
    assert!(
        saved.is_file(),
        "missing resolver byte blob for {content:?}"
    );
    saved
}

fn make_tsconfig(dir: &Path, paths_json: &str) -> TsConfig {
    let content = format!(r#"{{"compilerOptions": {{"paths": {}}}}}"#, paths_json);
    let p = dir.join("tsconfig.json");
    write(&p, &content);
    load_tsconfig(&p).unwrap()
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

fn scoped_resolution_candidates_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/scoped-resolution-candidates"),
    )
}

fn fixed_root_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/fixed-root-fast-path/root"),
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

fn symlinked_root_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlinked-root-tsconfig"),
    )
}

fn symlink_workspace_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    )
}

fn broken_child_boundary_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/broken-child-boundary"),
    )
}

fn trailing_directory_include_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/trailing-directory-include"),
    )
}

fn dotted_config_paths_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/dotted-config-paths"),
    )
}

fn extended_base_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/extended-base-ownership"),
    )
}

fn hidden_reference_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/hidden-reference"),
    )
}

fn auxiliary_ownership_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/auxiliary-ownership"),
    )
}

fn paths_precedence_tsconfig_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/paths-precedence"),
    )
}

mod aliases;
mod catalog_config;
mod catalog_ownership;
mod config_loading_and_relative;
mod resolver_cache;
mod scoped_resolution;

mod extends;
mod workspace_fallback;
