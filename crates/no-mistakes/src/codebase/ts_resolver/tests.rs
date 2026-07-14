// no-mistakes-disable-file rust-max-lines-per-file: legacy resolver coverage suite
use super::*;
use std::collections::HashSet;
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

fn write(path: &Path, content: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, content).unwrap();
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

// ── load_tsconfig ─────────────────────────────────────────────────────

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

// ── resolve_import — aliases ──────────────────────────────────────────

#[test]
fn resolves_alias_exact() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("lib").join("core.mts");
    write(&target, "");
    let tc = make_tsconfig(dir.path(), r#"{"@core": ["./lib/core"]}"#);
    let importer = dir.path().join("main.mts");
    assert_eq!(resolve_import("@core", &importer, &tc), Some(target));
}

#[test]
fn resolves_alias_wildcard() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("utils").join("helpers.mts");
    write(&target, "");
    let tc = make_tsconfig(dir.path(), r#"{"@utils/*": ["./utils/*"]}"#);
    let importer = dir.path().join("main.mts");
    assert_eq!(
        resolve_import("@utils/helpers", &importer, &tc),
        Some(target)
    );
}

#[test]
fn alias_wildcard_with_subpath() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("systems").join("emails").join("queues.mts");
    write(&target, "");
    let tc = make_tsconfig(dir.path(), r#"{"@systems/*": ["./systems/*"]}"#);
    let importer = dir.path().join("main.mts");
    assert_eq!(
        resolve_import("@systems/emails/queues", &importer, &tc),
        Some(target)
    );
}

#[test]
fn alias_nonexistent_returns_none() {
    let dir = TempDir::new().unwrap();
    let tc = make_tsconfig(dir.path(), r#"{"@utils/*": ["./utils/*"]}"#);
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("@utils/ghost", &importer, &tc).is_none());
}

#[test]
fn bare_npm_returns_none() {
    let dir = TempDir::new().unwrap();
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
    let dir = TempDir::new().unwrap();
    let tc = make_tsconfig(dir.path(), r#"{"*": ["./*"]}"#);
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("some-npm-pkg", &importer, &tc).is_none());
}

#[test]
fn import_resolver_uses_visible_file_set() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);

    assert_eq!(resolver.resolve("./utils", &importer), Some(target));
}

/// Regression test: `with_visible` (used by every `DepGraph` build and by
/// `server_routes`) must not disable the resolve cache. `resolve()`'s cache-hit
/// branch is a no-op when `cache_enabled` is false, so the "reuses/preserves"
/// tests below pass on identical *results* even with caching off — they don't
/// prove memoization happened. This asserts the cache is actually populated.
#[test]
fn import_resolver_with_visible_keeps_cache_enabled() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: HashSet<PathBuf> = [target].into();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);
    assert!(resolver.cache_enabled);

    resolver.resolve("./utils", &importer);

    assert_eq!(resolver.cache.len(), 1);
}

/// Regression test: calling `with_visible` on a resolver that already has
/// cached entries (resolved against the real filesystem, or an earlier
/// `visible` set) must not leak those stale answers into the new visibility
/// scope — `with_visible` consumes and returns `Self`, so a reused resolver's
/// `cache` would otherwise carry answers computed under different visibility.
#[test]
fn import_resolver_with_visible_clears_stale_cache_entries() {
    let dir = TempDir::new().unwrap();
    let target = normalize_path(&dir.path().join("src").join("utils.mts"));
    let importer = dir.path().join("src").join("main.mts");
    write(&target, "");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);
    assert_eq!(resolver.resolve("./utils", &importer), Some(target));

    let visible: HashSet<PathBuf> = HashSet::new();
    let resolver = resolver.with_visible(&visible);

    assert!(resolver.resolve("./utils", &importer).is_none());
}

#[test]
fn import_resolver_cache_reuses_present_result() {
    let dir = TempDir::new().unwrap();
    let target = normalize_path(&dir.path().join("src").join("utils.mts"));
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);

    assert_eq!(resolver.resolve("./utils", &importer), Some(target.clone()));
    assert_eq!(resolver.resolve("./utils", &importer), Some(target));
    assert!(resolver.resolve("./utils.mts", &importer).is_some());
    assert!(resolver.resolve("./missing.mts", &importer).is_none());
}

#[test]
fn import_resolver_cache_preserves_missing_result() {
    let dir = TempDir::new().unwrap();
    let importer = dir.path().join("src").join("main.mts");
    let target = dir.path().join("src").join("utils.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);

    assert!(resolver.resolve("./utils", &importer).is_none());
    write(&target, "");
    assert!(resolver.resolve("./utils", &importer).is_none());
}

#[test]
fn import_resolver_reports_exact_cached_work_for_hits_and_misses() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let resolver =
        ImportResolver::new_observed(&config, Some(observer.clone())).with_visible(&visible);

    assert_eq!(
        resolver.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert_eq!(resolver.resolve("./data.json", &importer), Some(target));
    assert!(resolver.resolve("./missing.json", &importer).is_none());
    assert!(resolver.resolve("./missing.json", &importer).is_none());

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 4);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.cache_hits"], 2);
    assert_eq!(work["resolver.resolved"], 1);
    assert_eq!(work["resolver.unresolved"], 1);
}

#[test]
fn import_resolver_single_flights_concurrent_hits_and_misses() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let resolver =
        ImportResolver::new_observed(&config, Some(observer.clone())).with_visible(&visible);
    let requests_per_key = 32;
    let barrier = std::sync::Barrier::new(requests_per_key * 2);

    std::thread::scope(|scope| {
        for request in 0..requests_per_key * 2 {
            let resolver = &resolver;
            let importer = &importer;
            let target = &target;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                if request % 2 == 0 {
                    assert_eq!(
                        resolver.resolve("./data.json", importer),
                        Some(target.clone())
                    );
                } else {
                    assert!(resolver.resolve("./missing.json", importer).is_none());
                }
            });
        }
    });

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 64);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.cache_hits"], 62);
    assert_eq!(work["resolver.resolved"], 1);
    assert_eq!(work["resolver.unresolved"], 1);
}

#[test]
fn import_resolver_session_reuses_only_exact_resolution_scopes() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let hidden = HashSet::new();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let first = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        first.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert!(first.resolve("./missing.json", &importer).is_none());
    let second = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        second.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert!(second.resolve("./missing.json", &importer).is_none());

    let hidden_scope = ImportResolver::new_in_session(&config, Some(&hidden), &session);
    assert!(hidden_scope.resolve("./data.json", &importer).is_none());
    let filesystem_scope = ImportResolver::new_in_session(&config, None, &session);
    assert_eq!(
        filesystem_scope.resolve("./data.json", &importer),
        Some(target)
    );

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 6);
    assert_eq!(work["resolver.computations"], 4);
    assert_eq!(work["resolver.unique_keys"], 4);
    assert_eq!(work["resolver.cache_hits"], 2);
    assert_eq!(work["resolver.resolved"], 2);
    assert_eq!(work["resolver.unresolved"], 2);
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

mod extends;
