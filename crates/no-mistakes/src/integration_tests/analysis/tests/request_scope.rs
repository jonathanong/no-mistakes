use super::*;

pub(super) fn collect_scoped(
    root: &std::path::Path,
    files: Vec<PathBuf>,
    plan: CheckFactPlan,
    config: &crate::config::v2::NoMistakesConfig,
) -> crate::codebase::check_facts::CheckFactMap {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.prepare_integration_fact_discovery(root, &files, config);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::check_facts::collect_prepared_check_facts(
        &session,
        root,
        (files, Vec::new()),
        Vec::new(),
        plan,
        None,
        sources,
    )
}

fn findings(
    root: &std::path::Path,
    config: &crate::config::v2::NoMistakesConfig,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Vec<crate::integration_tests::IntegrationFinding> {
    let analyses = facts
        .ts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .integration
                .clone()
                .map(|analysis| (path.clone(), analysis))
        })
        .collect();
    checks::check_suites(
        root,
        &test_support::configured_suites(root, config).unwrap(),
        &test_support::tsconfig_without_config(root),
        &analyses,
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        facts,
    )
    .unwrap()
}

#[test]
fn same_root_scoped_and_direct_batch_requests_keep_their_projections_in_both_orders() {
    use crate::codebase::check_facts::{
        collect_check_fact_batch_with_session, BatchCheckFactRequest,
    };
    for case in [
        "discovery-scope",
        "discovery-scope-errors",
        "discovery-scope-namespace-error",
    ] {
        let source = fixture().join(format!("../../{case}/fixture"));
        let saved = crate::test_support::materialize_saved_fixture(&source);
        let root = crate::codebase::ts_resolver::normalize_path(saved.path());
        let files = discover_files(&root, &[]);
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let plan = CheckFactPlan {
            integration: true,
            source: true,
            ..Default::default()
        };
        let full = collect_check_facts(&root, files.clone(), plan.clone());
        let scoped = collect_scoped(&root, files.clone(), plan.clone(), &config);
        let scope = config::test_discovery_scope(&root, &files, &config);
        let expected = findings(&root, &config, &full);
        if case == "discovery-scope" {
            assert_eq!(expected.len(), 7);
        } else {
            assert!(full.ts.values().any(|facts| facts.parse_error.is_some()));
        }
        for reverse in [false, true] {
            let session = crate::codebase::analysis_session::AnalysisSession::disabled();
            let request = |integration_discovery| BatchCheckFactRequest {
                root: root.clone(),
                files: files.clone(),
                graph_files: Vec::new(),
                plan: plan.clone(),
                playwright: None,
                sources: sources.clone(),
                integration_discovery,
            };
            let mut requests = vec![request(scope.clone()), request(FamilyPathScope::All)];
            if reverse {
                requests.reverse();
            }
            crate::ast::begin_parse_count(&root);
            let maps = collect_check_fact_batch_with_session(&session, requests);
            let parses = crate::ast::finish_parse_count(&root);
            assert!(!parses.is_empty());
            assert!(parses.values().all(|count| *count == 1), "{parses:#?}");
            for (index, actual) in maps.iter().enumerate() {
                let baseline = if (index == 0) != reverse {
                    &scoped
                } else {
                    &full
                };
                assert_eq!(findings(&root, &config, actual), expected);
                for (path, baseline) in &baseline.ts {
                    let actual = &actual.ts[path];
                    assert_eq!(actual.parse_error, baseline.parse_error);
                    match (&actual.integration, &baseline.integration) {
                        (Some(actual), Some(baseline)) => {
                            assert_eq!(actual.tests, baseline.tests, "{}", path.display());
                            assert_eq!(actual.exports, baseline.exports);
                            assert_eq!(actual.functions.len(), baseline.functions.len());
                        }
                        (None, None) => {}
                        _ => panic!("changed integration demand: {}", path.display()),
                    }
                }
            }
        }
    }
}

#[test]
fn declaring_prepared_scope_does_not_narrow_legacy_direct_collectors() {
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    assert_eq!(session.integration_fact_discovery(), &FamilyPathScope::All);
    session.prepare_integration_fact_discovery(&root, &files, &config);
    let plan = CheckFactPlan {
        integration: true,
        ..Default::default()
    };
    let full = collect_check_facts(&root, files.clone(), plan.clone());
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let direct = crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session, &root, (files, Vec::new()), plan, None, sources,
    );
    for (path, facts) in &full.ts {
        assert_eq!(
            direct.ts[path].integration.as_ref().unwrap().tests,
            facts.integration.as_ref().unwrap().tests
        );
    }
}

#[test]
fn legacy_only_scope_constructor_keeps_its_public_set_payload() {
    let constructor: fn(std::sync::Arc<std::collections::BTreeSet<PathBuf>>) -> FamilyPathScope =
        FamilyPathScope::Only;
    assert!(
        matches!(constructor(Default::default()), FamilyPathScope::Only(paths) if paths.is_empty())
    );
    assert_eq!(
        std::mem::size_of::<FamilyPathScope>(),
        std::mem::size_of::<usize>()
    );
}
