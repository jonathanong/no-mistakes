use super::*;

#[test]
fn resolution_candidates_cover_absolute_and_queue_compatibility_fallbacks() {
    let root = PathBuf::from("/resolver-candidate-fixture");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let absolute = PathBuf::from("/absolute/missing");
    let standard = ImportResolver::new(&tsconfig)
        .resolution_candidates(absolute.to_str().unwrap(), &root.join("src/main.ts"));
    assert!(standard.contains(&absolute.with_extension("ts")));
    assert!(standard.contains(&absolute.join("index.mts")));

    let queue = ImportResolver::new(&tsconfig).with_queue_compatibility(&root);
    let candidates = queue.resolution_candidates("workers/missing", &root.join("src/main.ts"));
    assert!(candidates.contains(&root.join("workers/missing.ts")));
    assert!(candidates.contains(&root.join("workers/missing/index.mts")));
    let source_candidates =
        queue.resolution_candidates("workers/missing.ts", &root.join("src/main.ts"));
    assert!(source_candidates.contains(&root.join("workers/missing.ts")));
    assert!(source_candidates.contains(&root.join("workers/missing.mts")));
}

// ── resolve_import — aliases ──────────────────────────────────────────

#[test]
fn aliases_follow_typescript_pattern_precedence_and_mapping_fallbacks() {
    let root = paths_precedence_tsconfig_fixture();
    let config = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let importer = root.join("src/entry.ts");
    let resolver = ImportResolver::new(&config);

    assert_eq!(
        resolver.resolve("feature/specific/detail", &importer),
        Some(root.join("src/longest-prefix/pecific/detail.ts"))
    );
    assert_eq!(
        resolver.resolve("exact/value", &importer),
        Some(root.join("src/exact.ts"))
    );
    assert_eq!(
        resolver.resolve("tie/value/detail", &importer),
        Some(root.join("src/first-tie/value/detail.ts"))
    );
    assert_eq!(
        resolver.resolve("replacement/value", &importer),
        Some(root.join("src/replacement/value.ts"))
    );
    assert_eq!(
        resolver.resolve("duplicate/value", &importer),
        Some(root.join("src/replacement/value.ts"))
    );
    assert_eq!(resolver.resolve("shadowed/value", &importer), None);
}

#[test]
fn aliases_preserve_declaration_ties_and_deleted_candidate_precedence() {
    let root = paths_precedence_tsconfig_fixture();
    let config = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let importer = root.join("src/entry.ts");
    let resolver = ImportResolver::new(&config);

    let patterns: Vec<_> = config
        .paths
        .iter()
        .map(|(pattern, _)| pattern.as_str())
        .collect();
    assert_eq!(&patterns[..3], ["*", "feature/*/detail", "feature/s*"]);
    assert!(!patterns.contains(&"stale-root/*"));
    assert!(!patterns.contains(&"stale-nested/*"));
    assert_eq!(
        patterns
            .iter()
            .filter(|pattern| **pattern == "duplicate/*")
            .count(),
        1
    );

    let candidates = resolver.resolution_candidates("shadowed/value", &importer);
    assert!(candidates.contains(&root.join("src/missing/value.ts")));
    assert!(!candidates.contains(&root.join("src/catch-all/shadowed/value.ts")));
}

#[test]
fn queue_compatibility_keeps_declaration_order_fallthrough() {
    let root = paths_precedence_tsconfig_fixture();
    let config = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let importer = root.join("src/entry.ts");
    let resolver = ImportResolver::new(&config).with_queue_compatibility(&root);

    assert_eq!(
        resolver.resolve("shadowed/value", &importer),
        Some(root.join("src/catch-all/shadowed/value.ts"))
    );
    let candidates = resolver.resolution_candidates("shadowed/value", &importer);
    assert!(candidates.contains(&root.join("src/catch-all/shadowed/value.ts")));
    assert!(candidates.contains(&root.join("src/missing/value.ts")));
}

#[test]
fn resolves_alias_exact() {
    let dir = saved_resolver_fixture("resolves-alias-exact");
    let target = dir.path().join("lib").join("core.mts");
    let tc = load_tsconfig(&dir.path().join("tsconfig.json")).unwrap();
    let importer = dir.path().join("main.mts");
    assert_eq!(resolve_import("@core", &importer, &tc), Some(target));
}

#[test]
fn resolves_alias_wildcard() {
    let dir = saved_resolver_fixture("resolves-alias-wildcard");
    let target = dir.path().join("utils").join("helpers.mts");
    let tc = load_tsconfig(&dir.path().join("tsconfig.json")).unwrap();
    let importer = dir.path().join("main.mts");
    assert_eq!(
        resolve_import("@utils/helpers", &importer, &tc),
        Some(target)
    );
}

#[test]
fn alias_wildcard_with_subpath() {
    let dir = saved_resolver_fixture("alias-wildcard-with-subpath");
    let target = dir.path().join("systems").join("emails").join("queues.mts");
    let tc = load_tsconfig(&dir.path().join("tsconfig.json")).unwrap();
    let importer = dir.path().join("main.mts");
    assert_eq!(
        resolve_import("@systems/emails/queues", &importer, &tc),
        Some(target)
    );
}

#[test]
fn alias_nonexistent_returns_none() {
    let dir = saved_resolver_fixture("alias-nonexistent-returns-none");
    let tc = load_tsconfig(&dir.path().join("tsconfig.json")).unwrap();
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("@utils/ghost", &importer, &tc).is_none());
}

#[test]
fn bare_npm_returns_none() {
    let dir = saved_resolver_fixture("bare-npm-returns-none");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("express", &importer, &tc).is_none());
    assert!(resolve_import("node:path", &importer, &tc).is_none());
}

#[test]
fn catch_all_nonexistent_returns_none() {
    let dir = saved_resolver_fixture("catch-all-nonexistent-returns-none");
    let tc = load_tsconfig(&dir.path().join("tsconfig.json")).unwrap();
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("some-npm-pkg", &importer, &tc).is_none());
}

// ── match_alias ───────────────────────────────────────────────────────

#[test]
fn match_alias_exact() {
    assert_eq!(match_alias("@core", "@core"), Some(String::new()));
    assert_eq!(match_alias("@core", "@other"), None);
}

#[test]
fn match_alias_wildcard() {
    assert_eq!(match_alias("@u/*", "@u/foo"), Some("foo".to_string()));
    assert_eq!(match_alias("@u/*", "@v/foo"), None);
}

#[test]
fn match_alias_wildcard_subpath() {
    assert_eq!(
        match_alias("@sys/*", "@sys/emails/queues"),
        Some("emails/queues".to_string())
    );
}
