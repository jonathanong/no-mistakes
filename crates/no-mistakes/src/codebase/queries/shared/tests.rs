use super::*;

#[test]
fn resolve_target_handles_absolute_and_cwd_fallback() {
    let cwd = std::env::current_dir().unwrap();

    // A relative file that does not exist under an (absolute) root falls back to
    // the current working directory.
    let fallback = resolve_target(
        Path::new("does-not-exist.ts"),
        Some(&cwd.join("nonexistent-subdir")),
        None,
    )
    .unwrap();
    assert_eq!(fallback.abs_file, cwd.join("does-not-exist.ts"));

    // An absolute file path is used as-is.
    let absolute_file = cwd.join("absolute.ts");
    let absolute = resolve_target(&absolute_file, None, None).unwrap();
    assert_eq!(absolute.abs_file, absolute_file);
}

#[test]
fn rel_str_strips_root_prefix() {
    let root = Path::new("/repo");
    assert_eq!(rel_str(Path::new("/repo/src/a.ts"), root), "src/a.ts");
    // A path outside the root is returned unchanged.
    assert_eq!(rel_str(Path::new("/other/a.ts"), root), "/other/a.ts");
}
