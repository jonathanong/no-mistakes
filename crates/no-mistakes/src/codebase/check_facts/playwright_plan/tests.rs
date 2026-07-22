use super::{
    BTreeMap, PlaywrightFactPlan, PlaywrightFactSelection, PlaywrightFileFactPlan,
    PlaywrightOccurrenceKey, PlaywrightSettingsKey,
};
use crate::playwright::playwright_tests::TestPolicy;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn selector_wrapper_resolution_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/selector-wrappers"),
    )
}

fn selector_wrapper_module_resolution() -> super::PlaywrightModuleResolution {
    let root = selector_wrapper_resolution_fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let paths = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, &root, &paths, &sources,
    )
    .unwrap();
    let workspace =
        crate::codebase::workspaces::load_indexed_from_source_store(&root, &sources).unwrap();
    super::PlaywrightModuleResolution::new(
        Arc::new(tsconfig),
        Arc::new(workspace),
        Arc::new(paths.iter().cloned().collect()),
    )
}

fn catalog_module_resolution(root: &Path) -> super::PlaywrightModuleResolution {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let paths = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
        root,
        &[root.to_path_buf()],
        &paths,
        &sources,
    );
    let workspace =
        crate::codebase::workspaces::load_indexed_from_source_store(root, &sources).unwrap();
    super::PlaywrightModuleResolution::with_catalog(
        Arc::new(catalog),
        Arc::new(workspace),
        Arc::new(paths.iter().cloned().collect()),
    )
}

fn symlinked_catalog_resolution_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    )
}

fn workspace_catalog_resolution_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

#[test]
fn wrapper_module_resolution_matches_aliases_and_nodenext_sources() {
    let root = selector_wrapper_resolution_fixture();
    let importing_file = root.join("tests/page.spec.ts");
    let resolution = selector_wrapper_module_resolution();

    assert!(resolution.modules_match(
        "./default-locator",
        "@fixture/default-locator",
        &importing_file,
    ));
    assert!(resolution.modules_match("./helpers.js", "./helpers", &importing_file));
}

#[test]
fn wrapper_module_resolution_rejects_distinct_unresolved_internal_modules() {
    let root = selector_wrapper_resolution_fixture();
    let importing_file = root.join("tests/page.spec.ts");
    let resolution = selector_wrapper_module_resolution();

    assert!(!resolution.modules_match("./missing-a", "./missing-b", &importing_file));
    assert!(!resolution.modules_match("./missing", "./missing", &importing_file));
    assert!(!resolution.modules_match("@missing/helper", "@missing/helper", &importing_file,));
}

#[test]
fn wrapper_module_resolution_keeps_external_packages_as_terminal_identities() {
    let root = selector_wrapper_resolution_fixture();
    let importing_file = root.join("tests/page.spec.ts");
    let resolution = selector_wrapper_module_resolution();

    assert!(resolution.modules_match("external-locators", "external-locators", &importing_file));
    assert!(!resolution.modules_match(
        "external-locators",
        "other-external-locators",
        &importing_file,
    ));
}

#[test]
fn catalog_wrapper_resolution_preserves_symlinked_alias_identity_without_rebuilding_scopes() {
    let root = symlinked_catalog_resolution_fixture();
    let importer = root.join("tests/dynamic-manual-mock.test.ts");
    let resolution = catalog_module_resolution(&root);

    for _ in 0..16 {
        assert!(resolution.modules_match("@linked/value", "../src/value", &importer));
    }

    // The facade shares the outer remapping universe. Repeated wrapper
    // comparisons only reuse its importer selection and owned scope resolver.
    assert_eq!(resolution.catalog_instrumentation(), Some((true, 1, 1, 2)));
}

#[test]
fn catalog_wrapper_resolution_does_not_treat_unresolved_workspace_packages_as_external() {
    let root = workspace_catalog_resolution_fixture();
    let importer = root.join("apps/web/src/entry.ts");
    let resolution = catalog_module_resolution(&root);

    // `@fixture/shared` is a known workspace package but this subpath is not
    // exported. It must not become a terminal external identity merely
    // because both configured and imported spellings are equal.
    assert!(!resolution.modules_match(
        "@fixture/shared/not-exported",
        "@fixture/shared/not-exported",
        &importer,
    ));
}

fn base_settings() -> crate::playwright::config::Settings {
    crate::playwright::config::Settings {
        frontend_root: "web".to_string(),
        playwright_configs: vec![PathBuf::from("b.ts"), PathBuf::from("a.ts")],
        project: None,
        test_include: vec!["b".to_string(), "a".to_string(), "b".to_string()],
        test_exclude: Vec::new(),
        ignore_routes: Vec::new(),
        rewrites: vec![
            crate::config::v2::schema::RewriteRule {
                source: "/a".to_string(),
                destination: "/b".to_string(),
            },
            crate::config::v2::schema::RewriteRule {
                source: "/b".to_string(),
                destination: "/c".to_string(),
            },
        ],
        navigation_helpers: vec!["z".to_string(), "a".to_string()],
        selector_wrappers: Vec::new(),
        selector_attributes: Vec::new(),
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: Vec::new(),
        selector_include: Vec::new(),
        selector_exclude: Vec::new(),
    }
}

impl PlaywrightFactPlan {
    pub(crate) fn from_settings(
        root: &Path,
        settings: crate::playwright::config::Settings,
        test_id_attributes_by_path: HashMap<PathBuf, Vec<String>>,
        scan_html_ids: bool,
        snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> anyhow::Result<Self> {
        let navigation_helpers = settings.navigation_helpers.clone();
        let selector_wrappers = settings.selector_wrappers.clone();
        let selector_attributes = settings.selector_attributes.clone();
        let component_selector_attributes = settings.component_selector_attributes.clone();
        let html_ids = settings.html_ids;
        let mut plan = Self::default();
        plan.add_source_settings(root, settings, scan_html_ids, snapshot)?;
        for (path, test_id_attributes) in test_id_attributes_by_path {
            plan.add_file(PlaywrightFactSelection {
                path,
                navigation_helpers: &navigation_helpers,
                selector_wrappers: &selector_wrappers,
                selector_attributes: &selector_attributes,
                component_selector_attributes: &component_selector_attributes,
                html_ids,
                test_id_attributes: &test_id_attributes,
                policy: TestPolicy::default(),
                demands_text_imports: true,
            });
        }
        Ok(plan)
    }

    pub(crate) fn set_app_source_files(&mut self, files: impl IntoIterator<Item = PathBuf>) {
        let files = Arc::new(
            files
                .into_iter()
                .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
                .collect::<HashSet<_>>(),
        );
        for plan in &mut self.source_plans {
            plan.app_source_files = Arc::clone(&files);
        }
    }
}

impl PlaywrightFileFactPlan {
    pub(crate) fn merged_test_id_attributes(&self) -> Vec<String> {
        self.variants
            .keys()
            .flat_map(|key| key.test_id_attributes.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn selector_extraction_count(&self) -> usize {
        self.variants.len()
    }
}

#[test]
fn occurrence_key_sorts_and_deduplicates_sequence_fields() {
    let key = PlaywrightOccurrenceKey::new(
        &["goB".to_string(), "goA".to_string(), "goB".to_string()],
        &[
            crate::config::v2::schema::PlaywrightSelectorWrapper {
                module: "@app/locators".to_string(),
                export: "getByTestId".to_string(),
                test_id_argument: 1,
            },
            crate::config::v2::schema::PlaywrightSelectorWrapper {
                module: "@app/locators".to_string(),
                export: "getByTestId".to_string(),
                test_id_argument: 1,
            },
        ],
        &["data-b".to_string(), "data-a".to_string()],
        &BTreeMap::from([
            ("propB".to_string(), "data-b".to_string()),
            ("propA".to_string(), "data-a".to_string()),
        ]),
        true,
        &[
            "data-b".to_string(),
            "data-a".to_string(),
            "data-b".to_string(),
        ],
    );

    assert_eq!(key.navigation_helpers, ["goA", "goB"]);
    assert_eq!(key.selector_wrappers.len(), 1);
    assert_eq!(key.selector_attributes, ["data-a", "data-b"]);
    assert_eq!(key.test_id_attributes, ["data-a", "data-b"]);
    assert_eq!(key.component_selector_attributes["propA"], "data-a");
    assert!(key.html_ids);
}

#[test]
fn settings_key_normalizes_set_fields_but_preserves_rewrite_order() {
    let first = base_settings();
    let mut equivalent = base_settings();
    equivalent.test_include.reverse();
    equivalent.navigation_helpers.reverse();
    assert_eq!(
        PlaywrightSettingsKey::new(&first),
        PlaywrightSettingsKey::new(&equivalent)
    );

    equivalent.playwright_configs.reverse();
    assert_ne!(
        PlaywrightSettingsKey::new(&first),
        PlaywrightSettingsKey::new(&equivalent)
    );

    equivalent.playwright_configs.reverse();
    equivalent.rewrites.reverse();
    assert_ne!(
        PlaywrightSettingsKey::new(&first),
        PlaywrightSettingsKey::new(&equivalent)
    );

    let mut wrapper_only = first.clone();
    wrapper_only.selector_wrappers = vec![crate::config::v2::schema::PlaywrightSelectorWrapper {
        module: "@app/locators".to_string(),
        export: "find".to_string(),
        test_id_argument: 0,
    }];
    assert_eq!(
        PlaywrightSettingsKey::new(&first),
        PlaywrightSettingsKey::new(&wrapper_only)
    );
}

#[test]
fn source_plans_coalesce_when_only_selector_wrappers_differ() {
    let first = base_settings();
    let mut wrapper_only = first.clone();
    wrapper_only.selector_wrappers = vec![crate::config::v2::schema::PlaywrightSelectorWrapper {
        module: "@app/locators".to_string(),
        export: "find".to_string(),
        test_id_argument: 0,
    }];
    let regexes = Arc::new(
        crate::playwright::selectors::compile_selector_regexes_with_html_ids(
            &[],
            &BTreeMap::new(),
            false,
        ),
    );
    let source_plan = |settings: crate::playwright::config::Settings, file: &str| {
        let settings_key = PlaywrightSettingsKey::new(&settings);
        super::PlaywrightSourceFactPlan {
            app_source_files: Arc::new(HashSet::from([PathBuf::from(file)])),
            selector_regexes: Arc::clone(&regexes),
            settings: Arc::new(settings),
            visible_files: Arc::new(HashSet::new()),
            scan_html_ids: false,
            settings_key,
        }
    };
    let mut plan = PlaywrightFactPlan::default();
    plan.merge_source_plan(source_plan(first, "a.tsx"));
    plan.merge_source_plan(source_plan(wrapper_only, "b.tsx"));

    assert_eq!(plan.source_plans.len(), 1);
    assert_eq!(plan.source_plans[0].app_source_files.len(), 2);
}
