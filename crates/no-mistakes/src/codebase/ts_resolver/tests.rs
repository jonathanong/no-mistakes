use super::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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

mod catalog_ownership;
mod resolver_cache;
mod scoped_resolution;
mod workspace_fallback;

#[test]
fn prepared_symbol_flows_keep_session_scoped_resolution() {
    for (name, source) in [
        ("pipeline", include_str!("../symbols/pipeline.rs")),
        (
            "impact_collect",
            include_str!("../symbols/impact_collect.rs"),
        ),
    ] {
        assert!(
            source.contains("ScopedImportResolver::from_lookup")
                || source.contains("ScopedImportResolver::new_in_session"),
            "{name} must resolve through a scoped session constructor"
        );
    }
}
