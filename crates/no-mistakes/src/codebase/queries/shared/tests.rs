use super::*;
use crate::codebase::ts_resolver::normalize_path;

fn query_fixture_root() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
    )
}

fn invalid_tsconfig_fixture_root() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis/query-invalid-tsconfig"),
    )
}

#[test]
fn resolve_target_handles_absolute_and_cwd_fallback() {
    // `Cargo.toml` exists in the crate directory (the test cwd) but not under the
    // bogus root, so input resolution falls back to the current directory.
    let cwd = std::env::current_dir().unwrap();

    let fallback = resolve_target(
        Path::new("Cargo.toml"),
        Some(&cwd.join("nonexistent-subdir")),
        None,
    )
    .unwrap();
    assert_eq!(fallback.abs_file, normalize_path(&cwd.join("Cargo.toml")));

    // An absolute file path is used as-is.
    let absolute_file = cwd.join("Cargo.toml");
    let absolute = resolve_target(&absolute_file, None, None).unwrap();
    assert_eq!(absolute.abs_file, normalize_path(&absolute_file));
}

#[test]
fn resolve_target_rejects_missing_file_or_directory() {
    let missing = resolve_target(Path::new("does-not-exist.ts"), None, None)
        .err()
        .unwrap();
    assert!(missing.to_string().contains("not a file"));
    // A directory is not a valid file target either.
    let dir = resolve_target(Path::new("src"), None, None).err().unwrap();
    assert!(dir.to_string().contains("not a file"));
}

#[test]
fn resolve_targets_share_the_request_visible_file_set() {
    let root = query_fixture_root();
    let targets = resolve_targets(
        &[PathBuf::from("consumer.ts"), PathBuf::from("broken.ts")],
        Some(&root),
        None,
    )
    .unwrap();

    // The repository-sized set is a request projection, not per-file work.
    assert!(Arc::ptr_eq(
        &targets[0].visible_files,
        &targets[1].visible_files
    ));
    assert!(std::ptr::eq(
        targets[0].visible_files(),
        targets[1].visible_files()
    ));
}

#[test]
fn reverse_preparation_is_lazy_and_reuses_the_target_tsconfig_source() {
    let root = query_fixture_root();
    let target = resolve_target(Path::new("consumer.ts"), Some(&root), None).unwrap();

    // Resolving a target is only path/inventory setup. It must not parse a
    // tsconfig, workspace manifest, or the repository-wide reverse universe.
    assert_eq!(target.sources.physical_read_count(), 0);
    let prepared = target.prepare_reverse().unwrap();
    assert!(!prepared.graph_files.indexable().is_empty());
    assert_eq!(target.sources.physical_read_count(), 1);

    // The later single-file resolver shares the catalog's source read instead
    // of rereading the same automatic tsconfig.
    assert!(!target.tsconfig().unwrap().paths.is_empty());
    assert_eq!(target.sources.physical_read_count(), 1);
}

#[test]
fn explicit_tsconfig_is_reused_for_single_file_and_reverse_queries() {
    let root = query_fixture_root();
    let target = resolve_target(
        Path::new("consumer.ts"),
        Some(&root),
        Some(Path::new("tsconfig.json")),
    )
    .unwrap();

    // An explicit config is authoritative for both the local resolver and the
    // reverse catalog, and the OnceLock keeps its parsed result stable.
    let first = target.tsconfig().unwrap();
    assert_eq!(first.paths[0].0, "@app/*");
    assert!(std::ptr::eq(first, target.tsconfig().unwrap()));

    let prepared = target.prepare_reverse().unwrap();
    assert_eq!(
        prepared.tsconfig_catalog.config_for(&target.abs_file),
        first
    );
}

#[test]
fn absolute_explicit_tsconfig_is_normalized_before_loading() {
    let root = query_fixture_root();
    // The redundant component makes this a lexical-normalization assertion,
    // not merely an assertion that absolute paths are accepted.
    let tsconfig = root.join("nested/../tsconfig.json");
    let target = resolve_target(Path::new("consumer.ts"), Some(&root), Some(&tsconfig)).unwrap();

    assert_eq!(
        target.explicit_tsconfig.as_deref(),
        Some(root.join("tsconfig.json").as_path())
    );
    assert_eq!(target.tsconfig().unwrap().paths[0].0, "@app/*");
}

#[test]
fn explicit_invalid_tsconfig_is_reported_consistently() {
    let root = invalid_tsconfig_fixture_root();
    let target = resolve_target(
        Path::new("entry.ts"),
        Some(&root),
        Some(Path::new("tsconfig.json")),
    )
    .unwrap();

    let first = target.tsconfig().unwrap_err().to_string();
    let second = target.tsconfig().unwrap_err().to_string();
    assert!(first.contains("tsconfig.json"), "{first}");
    assert_eq!(second, first);
}

#[test]
fn automatic_tsconfig_uses_an_empty_config_when_none_exists() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/no-tsconfig/fixture"),
    );
    let target = resolve_target(Path::new("entry.ts"), Some(&root), None).unwrap();

    // A TypeScript target without a project config still has a deterministic
    // empty resolver configuration. The source path is its anchor because that
    // is where discovery started.
    let config = target.tsconfig().unwrap();
    assert_eq!(config.dir, target.abs_file);
    assert!(config.paths.is_empty());
    assert!(config.base_url.is_none());
}

#[test]
fn automatic_invalid_tsconfig_falls_back_to_an_empty_config() {
    let root = invalid_tsconfig_fixture_root();
    let target = resolve_target(Path::new("entry.ts"), Some(&root), None).unwrap();

    // Automatic discovery is intentionally best effort: a malformed nearest
    // config cannot make a query that needs no aliases fail.
    let config = target.tsconfig().unwrap();
    assert_eq!(config.dir, root);
    assert_eq!(config.paths_dir, root);
    assert!(config.paths.is_empty());
    assert!(config.base_url.is_none());
}

#[test]
fn read_symbols_reports_the_source_path_for_parse_errors() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/forbidden-dependencies-parse-error/fixture"),
    );
    let target = resolve_target(Path::new("entrypoints/broken.mts"), Some(&root), None).unwrap();

    let error = read_symbols(&target.abs_file, &target.sources).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("extracting symbols from"), "{message}");
    assert!(message.contains("broken.mts"), "{message}");
}

#[test]
fn rel_str_strips_root_prefix_and_forward_slashes() {
    let root = Path::new("/repo");
    assert_eq!(rel_str(Path::new("/repo/src/a.ts"), root), "src/a.ts");
    // A path outside the root is returned unchanged.
    assert_eq!(rel_str(Path::new("/other/a.ts"), root), "/other/a.ts");
    // Backslash separators are normalized to forward slashes (Windows output).
    assert_eq!(rel_str(Path::new(r"/repo/src\a.ts"), root), "src/a.ts");
}
