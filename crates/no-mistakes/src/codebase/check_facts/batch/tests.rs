use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::collections::BTreeMap;
use std::path::Path;

#[cfg(feature = "test-instrumentation")]
mod historical;
mod removals;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/demand-builder/fixture"),
    )
}

fn request(
    root: &Path,
    files: &[&str],
    graph_files: &[&str],
    plan: CheckFactPlan,
) -> BatchCheckFactRequest {
    let files = files.iter().map(|p| root.join(p)).collect::<Vec<_>>();
    let graph_files = graph_files.iter().map(|p| root.join(p)).collect::<Vec<_>>();
    let inventory = Arc::new(FileInventory::from_paths(&files));
    BatchCheckFactRequest {
        root: root.to_path_buf(),
        files,
        graph_files,
        plan,
        playwright: None,
        sources: Arc::new(SourceStore::new(inventory)),
    }
}

#[test]
fn borrows_primary_and_graph_plans_owning_only_legacy_override() {
    let root = root();
    let helper = root.join("web/helper.ts");
    let payload = root.join("payload.txt");
    let requests = [request(
        &root,
        &["web/primary.ts", "web/./primary.ts"],
        &["web/primary.ts", "web/helper.ts", "web/graph.ts"],
        CheckFactPlan {
            source: true,
            symbols: true,
            legacy_symbol_paths: [helper.clone()].into_iter().collect(),
            production_import_paths: Arc::new([payload.clone()].into_iter().collect()),
            ..Default::default()
        },
    )];
    #[cfg(feature = "test-instrumentation")]
    historical::assert_parity(&requests);
    let mut slots = demands::graph_plans(&requests);
    let mut clones = 0;
    let demands = demands::demands_by_path::<true>(&requests, &mut slots, &mut clones);
    assert_eq!(demands.len(), 4);
    assert_eq!(clones, 1, "only the graph-only legacy target needs a clone");
    assert!(std::ptr::eq(
        demands[&root.join("web/primary.ts")][0].plan.as_ref(),
        &requests[0].plan
    ));
    assert!(demands[&payload][0].plan.as_ref().source);
    let legacy = demands[&helper][0].plan.as_ref();
    assert!(legacy.symbols && legacy.legacy_symbol_paths.contains(&helper));
    assert!(!legacy.source);
    assert!(!demands[&root.join("web/graph.ts")][0].plan.as_ref().source);
    assert_eq!(requests[0].plan.legacy_symbol_paths.len(), 1);
}

#[test]
fn playwright_override_clones_once_without_widening_other_requests() {
    let root = root();
    let primary = root.join("web/primary.ts");
    let helper = root.join("web/helper.ts");
    let mut first = request(
        &root,
        &["web/primary.ts"],
        &["web/helper.ts", "web/graph.ts"],
        CheckFactPlan {
            source: true,
            symbols: true,
            legacy_symbol_paths: [helper.clone()].into_iter().collect(),
            ..Default::default()
        },
    );
    let mut playwright = PlaywrightFactPlan::default();
    playwright.add_file(crate::codebase::check_facts::PlaywrightFactSelection {
        path: root.join("web/graph.ts"),
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_attributes: &[],
        component_selector_attributes: &BTreeMap::new(),
        html_ids: false,
        test_id_attributes: &[],
        policy: crate::playwright::playwright_tests::TestPolicy::default(),
        demands_text_imports: false,
    });
    playwright.set_source_files(vec![primary.clone(), helper.clone(), helper.clone()]);
    first.playwright = Some(playwright);
    let requests = [
        first,
        request(&root, &["web/primary.ts"], &[], CheckFactPlan::default()),
    ];
    #[cfg(feature = "test-instrumentation")]
    historical::assert_parity(&requests);
    let mut slots = demands::graph_plans(&requests);
    let mut clones = 0;
    let demands = demands::demands_by_path::<true>(&requests, &mut slots, &mut clones);
    assert_eq!(
        clones, 2,
        "legacy and imports reuse the same owned helper plan"
    );
    assert_eq!(demands[&primary].len(), 2);
    assert!(demands[&primary][0].plan.as_ref().graph.imports);
    assert!(demands[&primary][0].plan.as_ref().source);
    assert!(!demands[&primary][1].plan.as_ref().graph.imports);
    assert!(!requests[0].plan.graph.imports);
    let helper_plan = demands[&helper][0].plan.as_ref();
    assert!(helper_plan.symbols && helper_plan.graph.imports && !helper_plan.source);
    assert!(
        !demands[&root.join("web/graph.ts")][0]
            .plan
            .as_ref()
            .graph
            .imports
    );
}

#[test]
fn counted_false_keeps_counter_untouched_for_needed_and_unneeded_overrides() {
    let root = root();
    for already_imports in [false, true] {
        let mut request = request(
            &root,
            &["web/primary.ts"],
            &[],
            CheckFactPlan {
                graph: crate::codebase::ts_source::facts::TsFactPlan {
                    imports: already_imports,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let mut playwright = PlaywrightFactPlan::default();
        playwright.set_source_files(vec![root.join("web/primary.ts")]);
        request.playwright = Some(playwright);
        let requests = [request];
        #[cfg(feature = "test-instrumentation")]
        historical::assert_parity(&requests);
        let mut slots = demands::graph_plans(&requests);
        let mut clones = 7;
        let demands = demands::demands_by_path::<false>(&requests, &mut slots, &mut clones);
        assert_eq!(clones, 7);
        let plan = demands[&root.join("web/primary.ts")][0].plan.as_ref();
        assert!(plan.graph.imports);
        assert_eq!(std::ptr::eq(plan, &requests[0].plan), already_imports);
    }
}

#[test]
fn expired_request_does_not_construct_graph_plan_or_clone() {
    let root = root();
    let requests = [request(
        &root,
        &["web/primary.ts"],
        &[],
        CheckFactPlan::default(),
    )];
    #[cfg(feature = "test-instrumentation")]
    historical::assert_parity(&requests);
    let mut slots = demands::graph_plans(&requests);
    let _deadline =
        crate::invocation::install_test_deadline(std::time::Duration::from_nanos(1)).unwrap();
    assert!(crate::invocation::check_timeout().is_err());
    let mut clones = 0;
    let demands = demands::demands_by_path::<true>(&requests, &mut slots, &mut clones);
    assert!(demands.is_empty());
    drop(demands);
    assert!(slots.iter().all(Option::is_none));
    assert_eq!(clones, 0);
}

#[test]
fn graph_only_demands_retain_exact_request_local_call_site_scopes() {
    let root = root();
    let helper = root.join("web/helper.ts");
    let graph = root.join("web/graph.ts");
    let first_scope = Arc::new([helper.clone()].into_iter().collect());
    let second_scope = Arc::new([graph.clone()].into_iter().collect());
    let requests = [
        request(
            &root,
            &["web/primary.ts"],
            &["web/helper.ts", "web/graph.ts"],
            CheckFactPlan {
                call_site_paths: Arc::clone(&first_scope),
                ..Default::default()
            },
        ),
        request(
            &root,
            &[],
            &["web/helper.ts", "web/graph.ts"],
            CheckFactPlan {
                call_site_paths: Arc::clone(&second_scope),
                ..Default::default()
            },
        ),
    ];
    #[cfg(feature = "test-instrumentation")]
    historical::assert_parity(&requests);
    let mut slots = demands::graph_plans(&requests);
    let mut clones = 0;
    let demands = demands::demands_by_path::<true>(&requests, &mut slots, &mut clones);
    assert_eq!(clones, 0);
    for path in [&helper, &graph] {
        let variants = &demands[path];
        assert_eq!(variants.len(), 2);
        assert!(Arc::ptr_eq(
            &variants[0].plan.as_ref().call_site_paths,
            &first_scope
        ));
        assert!(Arc::ptr_eq(
            &variants[1].plan.as_ref().call_site_paths,
            &second_scope
        ));
        assert_eq!(variants[0].plan.as_ref().call_site_paths.len(), 1);
        assert_eq!(variants[1].plan.as_ref().call_site_paths.len(), 1);
    }
}
