use super::*;

#[test]
fn load_tsconfig_parses_paths() {
    let dir = saved_resolver_fixture("load-tsconfig-parses-paths");
    let tc = load_tsconfig(&dir.path().join("tsconfig.json")).unwrap();
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@utils/*");
}

#[test]
fn load_tsconfig_preserves_decoded_utf8_string_identity() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/string-identity");
    let root = crate::test_support::materialize_saved_fixture(&fixture);
    let path = root.path().join("valid.json");
    let config = load_tsconfig(&path).unwrap();
    // JSONC values are decoded UTF-8; marker-like and literal escape text
    // must remain distinct without OXC's lone-surrogate compatibility encoding.
    assert_eq!(
        config.paths,
        vec![
            ("@replacement/*".into(), vec!["./\u{fffd}/*".into()]),
            ("@marker/*".into(), vec!["./\u{fffd}d800/*".into()]),
            ("@escaped/*".into(), vec![r"./\uD800/*".into()]),
            ("@paired/*".into(), vec!["./\u{10000}/*".into()]),
            ("@plain/*".into(), vec!["./normal/*".into()]),
        ]
    );
    let (located, locations) = load_tsconfig_with_path_locations(&path).unwrap();
    assert_eq!(located, config);
    assert_eq!(locations.unwrap().lines["@replacement/*"], 4);
}

#[test]
fn load_tsconfig_rejects_lone_surrogates_before_collecting_paths() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/string-identity");
    let root = crate::test_support::materialize_saved_fixture(&fixture);
    for (name, reason) in [
        ("lone-high.json", "unpaired high surrogate"),
        ("lone-low.json", "unpaired low surrogate"),
    ] {
        let error = load_tsconfig(&root.path().join(name))
            .unwrap_err()
            .to_string();
        assert!(error.contains(name), "{error}");
        assert!(error.contains("parsing"), "{error}");
        assert!(error.contains(reason), "{error}");
    }
}

#[test]
fn load_tsconfig_empty_returns_defaults() {
    let dir = saved_resolver_fixture("load-tsconfig-empty-returns-defaults");
    let p = dir.path().join("tsconfig.json");
    let tc = load_tsconfig(&p).unwrap();
    assert!(tc.paths.is_empty());
}

#[test]
fn load_tsconfig_invalid_json_errors() {
    let dir = saved_resolver_fixture("load-tsconfig-invalid-json-errors");
    let p = dir.path().join("tsconfig.json");
    assert!(load_tsconfig(&p).is_err());
}

#[test]
fn load_tsconfig_missing_file_errors() {
    let dir = saved_resolver_fixture("load-tsconfig-missing-file-errors");
    assert!(load_tsconfig(&dir.path().join("tsconfig.json")).is_err());
}

// ── find_tsconfig ─────────────────────────────────────────────────────

#[test]
fn find_tsconfig_finds_in_dir() {
    let dir = saved_resolver_fixture("find-tsconfig-finds-in-dir");
    let p = dir.path().join("tsconfig.json");
    assert_eq!(find_tsconfig(dir.path()), Some(p));
}

#[test]
fn find_tsconfig_finds_in_parent() {
    let root = fixture("find-tsconfig-parent");
    let p = root.join("tsconfig.json");
    let child = root.join("sub").join("dir");
    assert_eq!(find_tsconfig(&child), Some(p));
}

#[test]
fn find_tsconfig_finds_from_file() {
    let dir = saved_resolver_fixture("find-tsconfig-finds-from-file");
    let tsc = dir.path().join("tsconfig.json");
    let file = dir.path().join("src").join("main.mts");
    assert_eq!(find_tsconfig(&file), Some(tsc));
}

// ── resolve_import — relative ─────────────────────────────────────────

#[test]
fn resolves_relative_with_extension() {
    let dir = saved_resolver_fixture("resolves-relative-with-extension");
    let target = dir.path().join("src").join("utils.mts");
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
    let dir = saved_resolver_fixture("resolves-relative-no-ext-tries-mts");
    let target = dir.path().join("src").join("utils.mts");
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
    let dir = saved_resolver_fixture("resolves-relative-no-ext-falls-back-to-ts");
    let target = dir.path().join("src").join("utils.ts");
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
    let root = fixture("relative-parent");
    let target = normalize_path(&root.join("lib.mts"));
    // `src` exists so ../lib.mts resolves through an existing parent. The
    // importer file itself is absent; `.gitkeep` only keeps that directory.
    let importer = root.join("src").join("main.mts");
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert_eq!(resolve_import("../lib.mts", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_index_fallback() {
    let dir = saved_resolver_fixture("resolves-relative-index-fallback");
    let target = dir.path().join("src").join("utils").join("index.mts");
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
    let dir = saved_resolver_fixture("relative-nonexistent-returns-none");
    let importer = dir.path().join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert!(resolve_import("./ghost", &importer, &tc).is_none());
}
