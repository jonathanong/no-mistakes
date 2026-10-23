use crate::codebase::check_facts::{
    collect_check_facts, configure_family_path_scopes, CheckFactPlan, FamilyPathScope,
};
use crate::codebase::ts_source::discover_files;
use crate::integration_tests::{checks, config, test_support};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/discovery-scope/fixture"),
    )
}

#[test]
fn explicit_suite_scope_preserves_helper_and_collision_findings_and_skips_walks() {
    let root = fixture();
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let mut plan = CheckFactPlan {
        integration: true,
        source: true,
        ..Default::default()
    };
    let full = collect_check_facts(&root, files.clone(), plan.clone());
    configure_family_path_scopes(&root, &files, &config, &sources, &mut plan).unwrap();
    let scoped = collect_check_facts(&root, files.clone(), plan.clone());
    let suites = test_support::configured_suites(&root, &config).unwrap();
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let report = |facts: &crate::codebase::check_facts::CheckFactMap| {
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
            &root,
            &suites,
            &test_support::tsconfig_without_config(&root),
            &analyses,
            &session,
        )
        .unwrap()
    };
    assert_eq!(report(&scoped), report(&full));
    let findings = report(&scoped);
    assert_eq!(findings.len(), 6, "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.test_name.as_deref() == Some("bridge")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.test_name.as_deref() == Some("collision before")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.test_name.as_deref() == Some("collision call")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.test_name.as_deref() == Some("collision export")),
        "{findings:#?}"
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.test_name.as_deref() == Some("collision after")),
        "{findings:#?}"
    );
    for relative in [
        "helpers/bridge.ts",
        "excluded/dense.ts",
        "selected/excluded.ts",
    ] {
        let path = root.join(relative);
        let narrow = scoped.ts[&path].integration.as_ref().unwrap();
        let wide = full.ts[&path].integration.as_ref().unwrap();
        assert!(
            narrow.discovery_statement_visits < wide.discovery_statement_visits,
            "{relative}: {} vs {}",
            narrow.discovery_statement_visits,
            wide.discovery_statement_visits
        );
        assert!(narrow.tests.is_empty());
    }
    for relative in [
        "helpers/collision-before.ts",
        "helpers/collision-call.ts",
        "helpers/collision-export.ts",
        "helpers/collision-after.ts",
        "selected/suite.test.ts",
        "selected/browser.spec.ts",
    ] {
        let path = root.join(relative);
        let narrow = scoped.ts[&path].integration.as_ref().unwrap();
        let wide = full.ts[&path].integration.as_ref().unwrap();
        assert_eq!(narrow.tests, wide.tests, "{relative}");
        if relative.starts_with("helpers/") {
            assert!(
                narrow.discovery_statement_visits > wide.discovery_statement_visits,
                "collision fallback must execute full discovery: {relative}"
            );
        } else {
            assert_eq!(
                narrow.discovery_statement_visits, wide.discovery_statement_visits,
                "{relative}"
            );
        }
    }
    // A direct fact request keeps the full discovery contract; an unrestricted
    // enabled consumer must also widen a merged request back to that contract.
    let mut direct = CheckFactPlan {
        integration: true,
        ..Default::default()
    };
    direct.include(plan.clone());
    assert_eq!(direct.integration_test_paths, FamilyPathScope::All);
    plan.include(CheckFactPlan {
        integration: true,
        ..Default::default()
    });
    assert_eq!(plan.integration_test_paths, FamilyPathScope::All);
}

#[test]
fn runner_derived_invalid_and_unconfigured_scopes_keep_full_discovery() {
    let root = fixture();
    let files = discover_files(&root, &[]);
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let explicit = config.clone();
    config
        .tests
        .vitest
        .projects
        .get_mut("selected")
        .unwrap()
        .include
        .clear();
    assert_eq!(
        config::test_discovery_scope(&root, &files, &config),
        FamilyPathScope::All
    );
    config = explicit.clone();
    for policy in config
        .tests
        .vitest
        .projects
        .values_mut()
        .chain(config.tests.playwright.projects.values_mut())
    {
        policy.include = vec!["**/*".into()];
        policy.exclude.clear();
    }
    assert_eq!(
        config::test_discovery_scope(&root, &files, &config),
        FamilyPathScope::All
    );
    config = explicit;
    config
        .tests
        .vitest
        .projects
        .get_mut("selected")
        .unwrap()
        .include = vec!["[".into()];
    assert_eq!(
        config::test_discovery_scope(&root, &files, &config),
        FamilyPathScope::All
    );
    config.tests.vitest.projects.clear();
    config.tests.playwright.projects.clear();
    assert_eq!(
        config::test_discovery_scope(&root, &files, &config),
        FamilyPathScope::All
    );
}

#[test]
fn test_discovery_scope_keeps_out_of_suite_parse_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &fixture().join("../../discovery-scope-errors/fixture"),
    );
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let mut plan = CheckFactPlan {
        integration: true,
        source: true,
        ..Default::default()
    };
    let full = collect_check_facts(&root, files.clone(), plan.clone());
    configure_family_path_scopes(&root, &files, &config, &sources, &mut plan).unwrap();
    let scoped = collect_check_facts(&root, files, plan);
    let suites = test_support::configured_suites(&root, &config).unwrap();
    let error = |facts: &crate::codebase::check_facts::CheckFactMap| {
        checks::fail_on_dropped_files(&root, &suites, facts)
            .unwrap_err()
            .to_string()
    };
    assert_eq!(error(&scoped), error(&full));
    assert!(error(&scoped).contains("unselected/broken.ts"));
}
