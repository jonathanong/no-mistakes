use super::*;

#[test]
fn load_tsconfig_parses_paths() {
    let dir = TempDir::new().unwrap();
    let tc = make_tsconfig(dir.path(), r#"{"@utils/*": ["./utils/*"]}"#);
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@utils/*");
}

#[test]
fn load_tsconfig_empty_returns_defaults() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{}");
    let tc = load_tsconfig(&p).unwrap();
    assert!(tc.paths.is_empty());
}

#[test]
fn load_tsconfig_invalid_json_errors() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{ bad json }");
    assert!(load_tsconfig(&p).is_err());
}

#[test]
fn load_tsconfig_missing_file_errors() {
    let dir = TempDir::new().unwrap();
    assert!(load_tsconfig(&dir.path().join("tsconfig.json")).is_err());
}

// ── find_tsconfig ─────────────────────────────────────────────────────

#[test]
fn find_tsconfig_finds_in_dir() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{}");
    assert_eq!(find_tsconfig(dir.path()), Some(p));
}

#[test]
fn find_tsconfig_finds_in_parent() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{}");
    let child = dir.path().join("sub").join("dir");
    std::fs::create_dir_all(&child).unwrap();
    assert_eq!(find_tsconfig(&child), Some(p));
}

#[test]
fn find_tsconfig_finds_from_file() {
    let dir = TempDir::new().unwrap();
    let tsc = dir.path().join("tsconfig.json");
    write(&tsc, "{}");
    let file = dir.path().join("src").join("main.mts");
    write(&file, "");
    assert_eq!(find_tsconfig(&file), Some(tsc));
}

// ── resolve_import — relative ─────────────────────────────────────────

#[test]
fn resolves_relative_with_extension() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils.mts", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_no_ext_tries_mts() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_no_ext_falls_back_to_ts() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.ts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_dotted_stem_by_appending_known_extension() {
    let root = fixture("dotted-stem");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/button.stories.tsx"));
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert_eq!(
        resolve_import("./button.stories", &importer, &tc),
        Some(target)
    );
}

#[test]
fn resolves_relative_explicit_non_javascript_extension() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert_eq!(resolve_import("./data.json", &importer, &tc), Some(target));
}

#[test]
fn unresolved_explicit_non_javascript_extension_does_not_append_ts_extension() {
    let root = fixture("explicit-css");
    let importer = root.join("src/main.mts");
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert!(resolve_import("./styles.css", &importer, &tc).is_none());
}

#[test]
fn resolves_relative_parent() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("lib.mts");
    write(&target, "");
    // Create the src directory so ../lib.mts resolves through an existing parent.
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    let importer = src.join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("../lib.mts", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_index_fallback() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils").join("index.mts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils", &importer, &tc), Some(target));
}

#[test]
fn relative_nonexistent_returns_none() {
    let dir = TempDir::new().unwrap();
    let importer = dir.path().join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert!(resolve_import("./ghost", &importer, &tc).is_none());
}
