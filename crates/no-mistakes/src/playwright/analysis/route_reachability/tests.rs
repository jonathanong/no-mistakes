use super::import_cache::{get_or_compute_route_imports, RouteImportCache};
use super::*;
use crate::playwright::config::Settings;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

fn settings(selector_include: Vec<String>) -> Settings {
    Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_attributes: vec!["data-pw".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include,
        selector_exclude: vec![],
    }
}

#[test]
fn route_reachability_honors_selector_include() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let route = routes::Route {
        file: root.join("web/app/page.tsx"),
        pattern: "/".to_string(),
    };

    let reachable = collect_route_reachable_files(
        &root,
        &settings(vec!["web/app/components/unreachable-*.tsx".to_string()]),
        &[route],
    )
    .expect("route reachability should collect");

    assert!(reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .is_some_and(BTreeSet::is_empty));
}

#[test]
fn route_reachability_resolves_tsconfig_alias_imports() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let route = routes::Route {
        file: root.join("web/app/page.tsx"),
        pattern: "/".to_string(),
    };

    let reachable =
        collect_route_reachable_files(&root, &settings(vec![]), &[route]).expect("collects");

    assert!(reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .is_some_and(|files| files.contains(&Arc::new(
            "web/app/components/discuss-button.tsx".to_string()
        ))));
    let files = reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .expect("route should have reachable files");
    assert!(files.contains(&Arc::new(
        "web/app/components/reexported-button.tsx".to_string()
    )));
    assert!(files.contains(&Arc::new(
        "web/app/components/export-all-button.tsx".to_string()
    )));
    assert!(!files.contains(&Arc::new(
        "web/app/components/type-only-button.tsx".to_string()
    )));
}

#[test]
fn route_reachability_includes_app_router_wrappers() {
    let root =
        crate::playwright::test_support::fixture_path(&["nextjs-selectors", "route-wrappers"]);
    let route = routes::Route {
        file: root.join("web/app/page.tsx"),
        pattern: "/".to_string(),
    };

    let reachable =
        collect_route_reachable_files(&root, &settings(vec![]), &[route]).expect("collects");

    let files = reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .expect("route should have reachable files");
    assert!(files.contains(&Arc::new(
        "web/app/components/layout-button.tsx".to_string()
    )));
    assert!(files.contains(&Arc::new(
        "web/app/components/template-button.tsx".to_string()
    )));
    assert!(!files.contains(&Arc::new("web/app/components/error-button.tsx".to_string())));
    assert!(!files.contains(&Arc::new(
        "web/app/components/not-found-button.tsx".to_string()
    )));
}

#[test]
fn route_reachability_uses_frontend_tsconfig_and_script_imports() {
    let root =
        crate::playwright::test_support::fixture_path(&["nextjs-selectors", "frontend-tsconfig"]);
    let route = routes::Route {
        file: root.join("web/app/page.tsx"),
        pattern: "/".to_string(),
    };

    let reachable =
        collect_route_reachable_files(&root, &settings(vec![]), &[route]).expect("collects");

    let files = reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .expect("route should have reachable files");
    assert!(files.contains(&Arc::new("web/app/components/alias-button.tsx".to_string())));
    assert!(files.contains(&Arc::new(
        "web/app/components/dynamic-button.tsx".to_string()
    )));
    assert!(files.contains(&Arc::new(
        "web/app/components/template-button.tsx".to_string()
    )));
}

#[test]
fn route_import_collection_uses_shared_cache_entries() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let route_file = root.join("web/app/page.tsx");
    let cached_import = root.join("web/app/components/cached.tsx");
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let import_cache = RouteImportCache::new();
    let cached = OnceLock::new();
    cached
        .set(Ok(Arc::new(vec![cached_import.clone()])))
        .unwrap();
    import_cache.insert(
        crate::codebase::ts_resolver::normalize_path(&route_file),
        Arc::new(cached),
    );

    let imports =
        collect_route_imports(&route_file, &resolver, &import_cache).expect("imports collect");

    assert_eq!(imports.as_ref(), &vec![cached_import]);
}

#[test]
fn route_import_collection_computes_shared_cache_entries_once() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let route_file = root.join("web/app/page.tsx");
    let import_cache = RouteImportCache::new();
    let compute_calls = AtomicUsize::new(0);

    (0..32_usize).into_par_iter().for_each(|_| {
        let imports =
            get_or_compute_route_imports(&import_cache, route_file.clone(), |_normalized_path| {
                compute_calls.fetch_add(1, Ordering::SeqCst);
                Ok(vec![root.join("web/app/components/discuss-button.tsx")])
            })
            .expect("imports should compute");
        assert_eq!(imports.len(), 1);
    });

    assert_eq!(compute_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn route_import_collection_serves_cached_paths_without_filesystem_lookup() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let missing_route_file = root.join("web/app/missing.tsx");
    let cached_import = root.join("web/app/components/discuss-button.tsx");
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let import_cache = RouteImportCache::new();
    let cached = OnceLock::new();
    cached
        .set(Ok(Arc::new(vec![cached_import.clone()])))
        .unwrap();
    import_cache.insert(
        crate::codebase::ts_resolver::normalize_path(&missing_route_file),
        Arc::new(cached),
    );

    let imports = collect_route_imports(&missing_route_file, &resolver, &import_cache)
        .expect("cached paths should not require filesystem metadata");

    assert_eq!(imports.as_ref(), &vec![cached_import]);
}

#[test]
fn route_reachability_surfaces_import_parse_errors() {
    let root =
        crate::playwright::test_support::fixture_path(&["react-traits-components", "bad-file"]);
    let route = routes::Route {
        file: root.join("app/components/Broken.tsx"),
        pattern: "/broken".to_string(),
    };

    let error = collect_route_reachable_files(&root, &settings(vec![]), &[route])
        .expect_err("malformed route files should surface import parse errors");
    assert!(error.to_string().contains("Broken.tsx"));
}
