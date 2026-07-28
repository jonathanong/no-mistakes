use super::*;
use crate::codebase::ts_resolver::normalize_path;

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
fn reverse_preparation_is_lazy_and_reuses_the_target_tsconfig_source() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries/fixture");
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
fn rel_str_strips_root_prefix_and_forward_slashes() {
    let root = Path::new("/repo");
    assert_eq!(rel_str(Path::new("/repo/src/a.ts"), root), "src/a.ts");
    // A path outside the root is returned unchanged.
    assert_eq!(rel_str(Path::new("/other/a.ts"), root), "/other/a.ts");
    // Backslash separators are normalized to forward slashes (Windows output).
    assert_eq!(rel_str(Path::new(r"/repo/src\a.ts"), root), "src/a.ts");
}
