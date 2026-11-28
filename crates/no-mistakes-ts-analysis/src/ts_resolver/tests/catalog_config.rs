use super::*;

#[test]
fn catalog_config_helpers_cover_valid_and_invalid_shapes() {
    let root = workspace_tsconfig_fixture();
    let path = root.join("tsconfig.json");
    assert!(extends_values(&serde_json::json!({}), &path)
        .unwrap()
        .is_empty());
    assert_eq!(
        extends_values(&serde_json::json!({"extends": "./base"}), &path).unwrap(),
        ["./base"]
    );
    assert!(extends_values(&serde_json::json!({"extends": ["./a", 1]}), &path).is_err());
    assert!(extends_values(&serde_json::json!({"extends": true}), &path).is_err());

    assert!(parse_paths(&serde_json::json!([]), &root).is_err());
    assert!(parse_paths(&serde_json::json!({"@x/*": "src/*"}), &root).is_err());
    assert!(parse_paths(&serde_json::json!({"@x/*": [1]}), &root).is_err());
    assert_eq!(
        parse_paths(&serde_json::json!({"@x/*": ["${configDir}/src/*"]}), &root,).unwrap()[0].1[0],
        format!("{}/src/*", root.display())
    );
    assert!(config_relative_path(&serde_json::json!(1), &root, "baseUrl").is_err());
    assert!(
        config_relative_path(&serde_json::json!("src"), &root, "baseUrl")
            .unwrap()
            .ends_with("src")
    );
    assert!(string_list(&serde_json::json!("src"), &path, "include").is_err());
    assert!(string_list(&serde_json::json!([1]), &path, "include").is_err());
    assert_eq!(
        string_list(&serde_json::json!(["src"]), &path, "include").unwrap(),
        ["src"]
    );
    assert!(reference_values(&serde_json::json!({}), &path).is_err());
    assert!(reference_values(&serde_json::json!([{}]), &path).is_err());
    assert!(reference_values(&serde_json::json!([1]), &path).is_err());
    assert_eq!(
        reference_values(&serde_json::json!(["./a", {"path": "./b"}]), &path,).unwrap(),
        ["./a", "./b"]
    );
}

#[test]
fn catalog_extends_merge_preserves_independent_fields() {
    let root = workspace_tsconfig_fixture();
    let mut first = EffectiveConfig::new(root.join("first.json"), root.clone());
    first
        .apply_own(
            &serde_json::json!({
                "compilerOptions": {"paths": {"@x/*": ["src/*"]}},
                "include": ["src/**/*.ts"]
            }),
            &root.join("first.json"),
            &root,
            &|value| Ok(root.join(value)),
        )
        .unwrap();
    let mut second = EffectiveConfig::new(root.join("second.json"), root.clone());
    second
        .apply_own(
            &serde_json::json!({"compilerOptions": {"baseUrl": ".", "allowJs": true}}),
            &root.join("second.json"),
            &root,
            &|value| Ok(root.join(value)),
        )
        .unwrap();
    let mut child = EffectiveConfig::new(root.join("child.json"), root.clone());
    child.inherit(first);
    child.inherit(second);
    let config = child.tsconfig();
    assert_eq!(config.paths[0].0, "@x/*");
    assert_eq!(config.base_url.as_deref(), Some(root.as_path()));
    assert!(child.matcher().allow_js);
}

#[test]
fn catalog_effective_config_rejects_invalid_compiler_and_project_fields() {
    let root = workspace_tsconfig_fixture();
    let path = root.join("invalid.json");
    for value in [
        serde_json::json!({"compilerOptions": []}),
        serde_json::json!({"compilerOptions": {"allowJs": "yes"}}),
        serde_json::json!({"compilerOptions": {"moduleResolution": 1}}),
        serde_json::json!({"files": "src/a.ts"}),
        serde_json::json!({"include": [1]}),
        serde_json::json!({"exclude": [1]}),
        serde_json::json!({"references": {}}),
    ] {
        let mut config = EffectiveConfig::new(path.clone(), root.clone());
        assert!(config
            .apply_own(&value, &path, &root, &|value| Ok(root.join(value)))
            .is_err());
    }
    let mut config = EffectiveConfig::new(path.clone(), root.clone());
    assert!(config
        .apply_own(
            &serde_json::json!({"references": [{"path": "./child"}]}),
            &path,
            &root,
            &|_| Err("reference failure".to_string()),
        )
        .is_err());
}

#[test]
fn catalog_matcher_respects_files_includes_excludes_and_out_dir() {
    let root = workspace_tsconfig_fixture();
    let explicit = root.join("explicit.ts");
    let outside_explicit = root.join("../outside-explicit.ts");
    let matcher = ConfigMatcher {
        dir: root.clone(),
        real_dir: root.clone(),
        files: Some(BTreeSet::from([explicit.clone(), outside_explicit.clone()])),
        includes: None,
        excludes: Vec::new(),
        out_dir: None,
        allow_js: false,
    };
    assert!(matcher.owns(&explicit));
    assert!(matcher.owns(&outside_explicit));
    assert!(!matcher.owns(&root.join("other.ts")));
    assert!(!matcher.owns(Path::new("/outside.ts")));

    let matcher = ConfigMatcher {
        dir: root.clone(),
        real_dir: root.clone(),
        files: None,
        includes: Some(vec![GlobRule::new(&root, "src/**/*.ts").unwrap()]),
        excludes: vec![GlobRule::new(&root, "src/excluded/**").unwrap()],
        out_dir: Some(root.join("dist")),
        allow_js: false,
    };
    assert!(matcher.owns(&root.join("src/ok.ts")));
    assert!(!matcher.owns(&root.join("src/excluded/no.ts")));
    assert!(!matcher.owns(&root.join("dist/output.ts")));
    assert!(!matcher.owns(&root.join("src/no.js")));
    assert!(!matcher.owns(&root.join("README.md")));
    let outside_include = GlobRule::new(&root, "../outside/**/*.ts").unwrap();
    assert!(outside_include.matches(&root.join("../outside/src/value.ts")));
    assert!(GlobRule::new(&root, "[").is_none());
    assert!(!GlobRule::new(&root, "")
        .unwrap()
        .matches(Path::new("/outside")));
}

#[test]
fn catalog_matcher_treats_exact_include_files_as_exact() {
    let root = exact_include_files_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let sibling = root.join("src/sibling.ts");
    let module = root.join("src/module.mts");
    let other_module = root.join("src/other.mts");
    let nested = root.join("directory/nested.ts");
    let visible = vec![
        config.clone(),
        entry.clone(),
        sibling.clone(),
        module.clone(),
        other_module.clone(),
        nested.clone(),
    ];
    let mut builder = CatalogBuilder::new(&root, std::slice::from_ref(&root), &visible, None, None);
    let matcher = builder.load_effective(&config).unwrap().matcher();

    assert!(matcher.owns(&entry));
    assert!(matcher.owns(&module));
    assert!(matcher.owns(&nested));
    assert!(!matcher.owns(&sibling));
    assert!(!matcher.owns(&other_module));
}

#[test]
fn catalog_matcher_treats_dot_include_as_the_config_directory() {
    let root = include_config_directory_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let nested = root.join("src/nested/value.ts");
    let script = root.join("scripts/setup.mts");
    let visible = vec![
        config.clone(),
        entry.clone(),
        nested.clone(),
        script.clone(),
    ];
    let mut builder = CatalogBuilder::new(&root, std::slice::from_ref(&root), &visible, None, None);
    let matcher = builder.load_effective(&config).unwrap().matcher();

    assert!(matcher.owns(&entry));
    assert!(matcher.owns(&nested));
    assert!(matcher.owns(&script));
}

#[test]
fn catalog_matcher_accepts_directory_includes_with_trailing_separators() {
    let root = trailing_directory_include_tsconfig_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let setup = root.join("scripts/setup.ts");
    let mut builder = CatalogBuilder::new(
        &root,
        std::slice::from_ref(&root),
        &[config.clone(), entry.clone(), setup.clone()],
        None,
        None,
    );
    let matcher = builder.load_effective(&config).unwrap().matcher();

    assert!(matcher.owns(&entry));
    assert!(matcher.owns(&setup));
}

#[test]
fn catalog_matcher_matches_absolute_config_dir_include_and_exclude_globs() {
    let root = absolute_glob_rules_fixture();
    let config = root.join("tsconfig.json");
    let included = root.join("src/included.ts");
    let excluded = root.join("src/excluded/value.ts");
    let mut builder = CatalogBuilder::new(
        &root,
        std::slice::from_ref(&root),
        &[config.clone(), included.clone(), excluded.clone()],
        None,
        None,
    );
    let matcher = builder.load_effective(&config).unwrap().matcher();

    // `${configDir}` expands before glob matching, so both rules are absolute.
    assert!(matcher.owns(&included));
    assert!(!matcher.owns(&excluded));
    assert!(!matcher.owns(&root.join("outside.ts")));
}

#[test]
fn catalog_builder_helpers_cover_fallback_resolution_shapes() {
    let root = workspace_tsconfig_fixture();
    let builder = CatalogBuilder::new(&root, &[], &[], None, None);
    assert!(builder.candidates().is_empty());
    assert_eq!(
        builder.resolve_config_value(&root, "./tsconfig").unwrap(),
        root.join("tsconfig.json")
    );
    assert_eq!(
        builder.resolve_config_value(&root, ".").unwrap(),
        root.join("tsconfig.json")
    );
    assert!(builder
        .resolve_package_extends(&root, "@missing/tsconfig")
        .is_err());

    let catalog = TsConfigCatalog::forced(&root, empty_config(&root), None);
    let resolver = ScopedImportResolver::unbounded(&catalog);
    assert!(resolver
        .resolve("./missing", &root.join("src/entry.ts"))
        .is_none());
}

// ── load_tsconfig ─────────────────────────────────────────────────────
