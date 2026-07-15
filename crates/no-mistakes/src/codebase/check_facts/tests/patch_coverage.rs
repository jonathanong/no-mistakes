use super::super::file::{collect_file_fact_variants_with_session, CheckFactVariant};
use super::{collect_file_facts, CheckFactPlan};
use crate::codebase::check_facts::{
    playwright_aggregate_facts, CheckFileFacts, PlaywrightSettingsKey,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn legacy_symbol_plan(path: &Path) -> CheckFactPlan {
    CheckFactPlan {
        symbols: true,
        legacy_symbol_paths: std::collections::HashSet::from([
            crate::codebase::ts_resolver::normalize_path(path),
        ]),
        ..CheckFactPlan::default()
    }
}

#[test]
fn aggregate_resolves_deferred_selectors_from_precollected_exports() {
    let page = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]);
    let exports_path = page.with_file_name("selectors.ts");
    let visible = std::collections::HashSet::from([page.clone(), exports_path.clone()]);
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-pw".to_string()],
        &BTreeMap::new(),
    );
    let source = std::fs::read_to_string(&page).unwrap();
    let deferred = crate::ast::with_program(&page, &source, |program, source| {
        crate::playwright::selectors::extract_app_selectors_from_program_from_visible_deferred(
            &page, source, program, &regexes, &visible,
        )
    })
    .unwrap();
    let exports_source = std::fs::read_to_string(&exports_path).unwrap();
    let static_exports = crate::ast::with_program(&exports_path, &exports_source, |program, _| {
        crate::playwright::selectors::collect_static_export_values(program)
    })
    .unwrap();
    let settings = crate::playwright::config::Settings {
        frontend_root: String::new(),
        playwright_configs: Vec::new(),
        project: None,
        test_include: Vec::new(),
        test_exclude: Vec::new(),
        ignore_routes: Vec::new(),
        rewrites: Vec::new(),
        navigation_helpers: Vec::new(),
        selector_wrappers: Vec::new(),
        selector_attributes: vec!["data-pw".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: Vec::new(),
        selector_include: Vec::new(),
        selector_exclude: Vec::new(),
    };
    let settings_key = PlaywrightSettingsKey::new(&settings);
    let facts = std::collections::HashMap::from([
        (
            page,
            CheckFileFacts {
                playwright_app_selectors: std::collections::HashMap::from([(
                    (settings_key.clone(), false),
                    deferred,
                )]),
                ..Default::default()
            },
        ),
        (
            exports_path,
            CheckFileFacts {
                playwright_static_exports: Some(static_exports),
                ..Default::default()
            },
        ),
    ]);

    let (selectors, _) = playwright_aggregate_facts(&facts);
    let resolved = selectors
        .get(&(settings_key, false))
        .expect("settings cache is populated")
        .clone()
        .expect("selector aggregation succeeds");
    let mut values = resolved
        .iter()
        .map(|selector| match &selector.value {
            crate::playwright::selectors::AppSelectorValue::Exact(value) => value.clone(),
            value => panic!("expected an exact resolved selector, got {value:?}"),
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();

    assert_eq!(
        values,
        [
            "imported-const",
            "imported-fn-val",
            "imported-obj-a",
            "imported-obj-b",
        ]
    );
}

#[test]
fn legacy_symbol_facts_recover_symbols_with_a_parse_diagnostic() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/symbols-output/fixture"),
    );
    let file = root.join("src/recoverable-diagnostic.mts");
    let facts = collect_file_facts(&root, &file, &legacy_symbol_plan(&file), None)
        .expect("recoverable legacy parse retains facts");

    assert!(facts.parsed);
    assert!(facts.parse_error.is_some());
    assert!(facts.symbols.is_some());
    assert!(facts.ts.symbols.is_some());
}

#[test]
fn legacy_symbol_facts_retain_a_meaningful_fatal_parse_error() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/napi/analyze-project-legacy-symbol-panic"),
    );
    let file = root.join("invalid.ts");
    let facts = collect_file_facts(&root, &file, &legacy_symbol_plan(&file), None)
        .expect("fatal legacy parse retains error facts");
    let error = facts
        .parse_error
        .as_deref()
        .expect("fatal legacy parse error is recorded");

    assert!(
        error.contains("failed to parse TypeScript source"),
        "{error}"
    );
    assert_eq!(facts.ts.parse_error.as_deref(), Some(error));
    assert!(facts.symbols.is_none());
}

#[test]
fn collect_file_facts_retains_prepared_runner_config_parse_errors() {
    use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/parse-errors/fixture"),
    );
    let file = root.join("vitest.syntax-error.mts");
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.syntax-error.mts".to_string()));
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            integration_suites: std::collections::BTreeMap::from([(
                "integration".to_string(),
                Vec::new(),
            )]),
            ..Default::default()
        },
    );
    let runner_configs =
        crate::integration_tests::prepare_runner_configs(&root, &config, &visible, &tsconfig);

    let plan = CheckFactPlan {
        integration_runner_configs: Some(std::sync::Arc::new(runner_configs)),
        ..Default::default()
    };
    let facts = collect_file_facts(&root, &file, &plan, None)
        .expect("prepared runner config parse error is retained as facts");

    assert!(facts.integration_runner_config.is_some());
    assert!(facts
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("vitest.syntax-error.mts")));

    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let batched = collect_file_fact_variants_with_session(
        &session,
        &file,
        &[CheckFactVariant {
            root: &root,
            plan: &plan,
            playwright: None,
        }],
    )
    .into_iter()
    .next()
    .flatten()
    .expect("batched prepared runner config parse error is retained as facts");
    assert!(batched.integration_runner_config.is_some());
}
