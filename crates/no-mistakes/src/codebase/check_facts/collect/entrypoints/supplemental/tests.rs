use super::*;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::TsFactPlan;
use crate::codebase::ts_source::FileInventory;

fn call_site_plan() -> TsFactPlan {
    TsFactPlan {
        call_sites: true,
        ..Default::default()
    }
}

#[test]
fn finite_call_source_collects_one_domain_visitor_amid_unrelated_graph_facts() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped"),
    );
    let selected = root.join("generated/schedules.test.mts");
    let registry = root.join("registry.mts");
    let unrelated = root.join("src/lazy.mts");
    let paths = vec![selected.clone(), registry.clone(), unrelated.clone()];
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &paths,
    ))));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        collect_check_facts_with_supplemental_call_sites(
            &session,
            &root,
            (vec![registry.clone(), unrelated.clone()], Vec::new()),
            vec![selected.clone()],
            CheckFactPlan {
                graph: TsFactPlan::imports(),
                ..Default::default()
            },
            None,
            sources,
        )
    });
    assert_eq!(facts.stats.files_parsed, 3);
    assert_eq!(
        observer.snapshot().work.get("call_sites.collect_files"),
        Some(&1)
    );
    assert!(!facts.graph_plan().call_sites);
    assert!(facts.covers_ts_fact_plan_for_file(&selected, call_site_plan()));
    assert!(!facts.covers_ts_fact_plan_for_file(&registry, call_site_plan()));
    assert!(!facts.covers_ts_fact_plan_for_file(&unrelated, call_site_plan()));
    assert!(facts
        .get_ts_facts(&selected)
        .is_some_and(|facts| !facts.call_sites.is_empty()));
}

#[test]
fn finite_call_source_in_graph_only_scope_keeps_its_single_parse_facts() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped"),
    );
    let selected = root.join("generated/schedules.test.mts");
    let registry = root.join("registry.mts");
    let paths = vec![selected.clone(), registry.clone()];
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &paths,
    ))));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        collect_check_facts_with_supplemental_call_sites(
            &session,
            &root,
            (vec![registry.clone()], vec![selected.clone()]),
            Vec::new(),
            CheckFactPlan {
                graph: TsFactPlan::imports(),
                call_site_paths: Arc::new(vec![selected.clone()]),
                ..Default::default()
            },
            None,
            sources,
        )
    });
    assert_eq!(facts.stats.files_parsed, 2);
    assert_eq!(
        observer.snapshot().work.get("call_sites.collect_files"),
        Some(&1)
    );
    assert!(!facts.graph_plan().call_sites);
    assert!(facts.covers_ts_fact_plan_for_file(&selected, call_site_plan()));
    assert!(facts
        .get_ts_facts(&selected)
        .is_some_and(|facts| !facts.call_sites.is_empty()));
}

#[test]
fn graph_owned_call_site_demand_still_collects_all_graph_files() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped"),
    );
    let selected = root.join("generated/schedules.test.mts");
    let unrelated = root.join("registry.mts");
    let paths = vec![selected.clone(), unrelated.clone()];
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &paths,
    ))));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        collect_check_facts_with_supplemental_call_sites(
            &session,
            &root,
            (vec![unrelated.clone()], vec![selected.clone()]),
            Vec::new(),
            CheckFactPlan {
                graph: call_site_plan(),
                call_site_paths: Arc::new(vec![selected.clone()]),
                ..Default::default()
            },
            None,
            sources,
        )
    });
    assert_eq!(
        observer.snapshot().work.get("call_sites.collect_files"),
        Some(&2)
    );
    assert!(facts.covers_ts_fact_plan(call_site_plan()));
    assert!(facts.covers_ts_fact_plan_for_file(&unrelated, call_site_plan()));
}

#[test]
fn supplemental_call_source_keeps_its_richer_production_import_facts() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let call_source = normalize_path(&root.join("generated/schedules.test.mts"));
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&call_source),
    ))));
    let plan = CheckFactPlan {
        production_import_paths: Arc::new([call_source.clone()].into_iter().collect()),
        graph: TsFactPlan {
            imports: true,
            call_sites: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let facts = collect_check_facts_with_supplemental_call_sites(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        &root,
        (Vec::new(), Vec::new()),
        vec![call_source.clone()],
        plan,
        None,
        sources,
    );
    let file = facts.ts.get(&call_source).expect("union fact entry");
    assert_eq!(facts.files(), &[call_source]);
    assert!(
        file.production_imports.is_some(),
        "production import projection lost"
    );
    assert!(file
        .ts
        .imports
        .iter()
        .any(|import| import.specifier == "../src/lazy.mts"));
    assert!(
        !file.ts.call_sites.is_empty(),
        "call sites must remain available"
    );
}

#[test]
fn primary_plan_without_call_sites_still_keeps_supplemental_call_facts() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let call_source = normalize_path(&root.join("generated/schedules.test.mts"));
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&call_source),
    ))));
    // The primary plan omits call sites. This path is also a production-import
    // file, so the supplemental entry must request call sites or they are dropped.
    let plan = CheckFactPlan {
        production_import_paths: Arc::new([call_source.clone()].into_iter().collect()),
        graph: TsFactPlan {
            imports: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let facts = collect_check_facts_with_supplemental_call_sites(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        &root,
        (Vec::new(), Vec::new()),
        vec![call_source.clone()],
        plan,
        None,
        sources,
    );
    let file = facts.ts.get(&call_source).expect("union fact entry");
    assert!(
        file.production_imports.is_some(),
        "production import projection lost"
    );
    assert!(file
        .ts
        .imports
        .iter()
        .any(|import| import.specifier == "../src/lazy.mts"));
    assert!(
        !file.ts.call_sites.is_empty(),
        "call sites dropped when the primary plan omitted them"
    );
}

#[test]
fn playwright_only_test_source_keeps_supplemental_call_sites() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let call_source = normalize_path(&root.join("generated/schedules.test.mts"));
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&call_source),
    ))));
    let mut playwright = PlaywrightFactPlan::default();
    playwright.add_file(crate::codebase::check_facts::PlaywrightFactSelection {
        path: call_source.clone(),
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_attributes: &[],
        component_selector_attributes: &Default::default(),
        html_ids: false,
        test_id_attributes: &[],
        policy: Default::default(),
        demands_text_imports: false,
    });
    let facts = collect_check_facts_with_supplemental_call_sites(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        &root,
        (Vec::new(), Vec::new()),
        vec![call_source.clone()],
        CheckFactPlan::default(),
        Some(playwright),
        sources,
    );
    let file = facts
        .ts
        .get(&call_source)
        .expect("Playwright-only fact entry");
    assert!(file.playwright.is_some(), "Playwright test facts lost");
    assert!(!file.ts.call_sites.is_empty(), "configured call facts lost");
}

#[test]
fn supplemental_read_failure_remains_addressable_without_widening_scopes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let missing = root.join("generated/missing.mts");
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        std::slice::from_ref(&missing),
    ))));
    let facts = collect_check_facts_with_supplemental_call_sites(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        &root,
        (Vec::new(), Vec::new()),
        vec![missing.clone()],
        CheckFactPlan::default(),
        None,
        sources,
    );
    let file = facts.ts.get(&missing).expect("I/O failure fact");
    assert!(file
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
    // Discovered, parsed, and parse errors must count the same supplemental file.
    assert_eq!(
        (
            facts.stats.files_discovered,
            facts.stats.files_parsed,
            facts.stats.parse_errors,
        ),
        (1, 0, 1)
    );
    assert!(facts.files().is_empty());
    assert!(facts.graph_file_universe().is_empty());
}
