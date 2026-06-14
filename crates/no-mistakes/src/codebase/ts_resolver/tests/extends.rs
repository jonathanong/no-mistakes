use super::*;

// ── load_tsconfig extends ────────────────────────────────────────────

#[test]
fn load_tsconfig_follows_extends_relative() {
    let dir = TempDir::new().unwrap();
    let base_p = dir.path().join("tsconfig.base.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"paths": {"@utils/*": ["./utils/*"]}}}"#,
    );
    let child_p = dir.path().join("tsconfig.json");
    write(&child_p, r#"{"extends": "./tsconfig.base.json"}"#);

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@utils/*");
    assert_eq!(tc.paths_dir, dir.path().to_path_buf());
}

#[test]
fn load_tsconfig_follows_extends_from_subdir() {
    let root = TempDir::new().unwrap();
    let base_p = root.path().join("tsconfig.base.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"paths": {"@core/*": ["./packages/core/src/*"]}}}"#,
    );
    let sub = root.path().join("apps").join("web");
    std::fs::create_dir_all(&sub).unwrap();
    let child_p = sub.join("tsconfig.json");
    write(&child_p, r#"{"extends": "../../tsconfig.base.json"}"#);

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.dir, sub);
    assert_eq!(tc.paths_dir, root.path().to_path_buf());
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@core/*");
}

#[test]
fn load_tsconfig_child_paths_override_extends() {
    let dir = TempDir::new().unwrap();
    let base_p = dir.path().join("tsconfig.base.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"paths": {"@base/*": ["./base/*"]}}}"#,
    );
    let child_p = dir.path().join("tsconfig.json");
    write(
        &child_p,
        r#"{"extends": "./tsconfig.base.json", "compilerOptions": {"paths": {"@child/*": ["./child/*"]}}}"#,
    );

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@child/*");
    assert_eq!(tc.paths_dir, dir.path().to_path_buf());
}

#[test]
fn load_tsconfig_inherits_base_url_without_paths() {
    let dir = TempDir::new().unwrap();
    let base_p = dir.path().join("tsconfig.base.json");
    write(&base_p, r#"{"compilerOptions": {"baseUrl": "."}}"#);
    let child_p = dir.path().join("tsconfig.json");
    write(&child_p, r#"{"extends": "./tsconfig.base.json"}"#);
    let target = dir.path().join("lib").join("thing.mts");
    write(&target, "");

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.base_url, Some(dir.path().to_path_buf()));
    assert_eq!(
        resolve_import("lib/thing", &dir.path().join("src").join("main.mts"), &tc),
        Some(target)
    );
}

#[test]
fn load_tsconfig_child_paths_override_extends_but_inherit_base_url() {
    let dir = TempDir::new().unwrap();
    let base_p = dir.path().join("tsconfig.base.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"baseUrl": ".", "paths": {"@base/*": ["./base/*"]}}}"#,
    );
    let child_p = dir.path().join("tsconfig.json");
    write(
        &child_p,
        r#"{"extends": "./tsconfig.base.json", "compilerOptions": {"paths": {"@child/*": ["./child/*"]}}}"#,
    );

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@child/*");
    assert_eq!(tc.base_url, Some(dir.path().to_path_buf()));
}

#[test]
fn load_tsconfig_extends_missing_target_errors() {
    let dir = TempDir::new().unwrap();
    let child_p = dir.path().join("tsconfig.json");
    write(&child_p, r#"{"extends": "./nonexistent.json"}"#);
    assert!(load_tsconfig(&child_p).is_err());
}

#[test]
fn load_tsconfig_extends_cycle_errors() {
    let dir = TempDir::new().unwrap();
    let a_p = dir.path().join("a.json");
    let b_p = dir.path().join("b.json");
    write(&a_p, r#"{"extends": "./b.json"}"#);
    write(&b_p, r#"{"extends": "./a.json"}"#);
    assert!(load_tsconfig(&a_p).is_err());
}

#[test]
fn load_tsconfig_extends_npm_package_skipped_gracefully() {
    // npm-package extends cannot be resolved without node_modules; degrade to empty paths.
    let dir = TempDir::new().unwrap();
    let child_p = dir.path().join("tsconfig.json");
    write(&child_p, r#"{"extends": "@scope/tsconfig/base"}"#);
    let tc = load_tsconfig(&child_p).unwrap();
    assert!(tc.paths.is_empty());
}

#[test]
fn load_tsconfig_follows_extends_array() {
    let dir = TempDir::new().unwrap();
    let base_p = dir.path().join("tsconfig.base.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"paths": {"@base/*": ["./base/*"]}}}"#,
    );
    let child_p = dir.path().join("tsconfig.json");
    // TS 5.0+ array extends — rightmost entry with paths wins
    write(&child_p, r#"{"extends": ["./tsconfig.base.json"]}"#);

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@base/*");
    assert_eq!(tc.paths_dir, dir.path().to_path_buf());
}

#[test]
fn load_tsconfig_extends_array_inherits_paths_and_base_url_independently() {
    let dir = TempDir::new().unwrap();
    let paths_p = dir.path().join("tsconfig.paths.json");
    write(
        &paths_p,
        r#"{"compilerOptions": {"paths": {"@base/*": ["./base/*"]}}}"#,
    );
    let base_url_p = dir.path().join("tsconfig.base-url.json");
    write(&base_url_p, r#"{"compilerOptions": {"baseUrl": "."}}"#);
    let child_p = dir.path().join("tsconfig.json");
    write(
        &child_p,
        r#"{"extends": ["./tsconfig.paths.json", "./tsconfig.base-url.json"]}"#,
    );

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@base/*");
    assert_eq!(tc.paths_dir, dir.path().to_path_buf());
    assert_eq!(tc.base_url, Some(dir.path().to_path_buf()));
}

#[test]
fn load_tsconfig_extends_directory_appends_tsconfig_json() {
    let dir = TempDir::new().unwrap();
    let subdir = dir.path().join("base");
    std::fs::create_dir_all(&subdir).unwrap();
    let base_p = subdir.join("tsconfig.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"paths": {"@lib/*": ["./lib/*"]}}}"#,
    );
    let child_p = dir.path().join("tsconfig.json");
    write(&child_p, r#"{"extends": "./base"}"#);

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@lib/*");
}

#[test]
fn load_tsconfig_extends_extensionless_file_appends_json() {
    let dir = TempDir::new().unwrap();
    let base_p = dir.path().join("base.json");
    write(
        &base_p,
        r#"{"compilerOptions": {"paths": {"@lib/*": ["./lib/*"]}}}"#,
    );
    let child_p = dir.path().join("tsconfig.json");
    write(&child_p, r#"{"extends": "./base"}"#);

    let tc = load_tsconfig(&child_p).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@lib/*");
}

#[test]
fn load_tsconfig_extends_array_nonstring_entry_errors() {
    let dir = TempDir::new().unwrap();
    let child_p = dir.path().join("tsconfig.json");
    // TypeScript rejects non-string entries in the extends array.
    write(&child_p, r#"{"extends": ["./tsconfig.base.json", 42]}"#);
    assert!(load_tsconfig(&child_p).is_err());
}

#[test]
fn load_tsconfig_extends_nonstring_toplevel_errors() {
    let dir = TempDir::new().unwrap();
    let child_p = dir.path().join("tsconfig.json");
    // TypeScript rejects a non-string/array extends value (e.g. a number).
    write(&child_p, r#"{"extends": 123}"#);
    assert!(load_tsconfig(&child_p).is_err());
}

#[test]
fn resolver_caches_results_and_returns_consistent_value() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("src").join("main.mts");
    let dep = dir.path().join("src").join("dep.mts");
    write(&file, "");
    write(&dep, "");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);
    // First call populates the cache.
    let first = resolver.resolve("./dep.mts", &file);
    // Second call must hit the cache and return the same result.
    let second = resolver.resolve("./dep.mts", &file);
    assert_eq!(first, second);
    assert_eq!(first, Some(dep));
}

// ── normalize_path ────────────────────────────────────────────────────

#[test]
fn normalize_path_preserves_parent_dir_at_root() {
    let p = normalize_path(Path::new("/a/../../b"));
    let s = p.to_string_lossy();
    assert!(s.contains("b"), "path should still reach b: {s}");
    assert!(!s.contains("a"), "a should have been popped: {s}");
}

#[test]
fn normalize_path_double_parent_from_root() {
    let p = normalize_path(Path::new("/../../b"));
    let s = p.to_string_lossy();
    assert!(s.contains("b"));
}

#[test]
fn normalize_path_drops_current_dir_components() {
    assert_eq!(normalize_path(Path::new("./a/./b")), Path::new("a/b"));
}

#[test]
fn match_alias_captures_wildcard_segment() {
    assert_eq!(match_alias("@/*", "@/foo"), Some("foo".to_string()));
}
