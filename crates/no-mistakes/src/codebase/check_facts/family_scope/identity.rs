use super::{configure_family_path_scopes, FamilyPathScope};
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_source::discover_files;
use std::path::{Path, PathBuf};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(rel)
}

fn queue_plan(root: &Path) -> CheckFactPlan {
    let config = crate::config::v2::load_v2_config(root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    CheckFactPlan {
        queue: true,
        queue_factory_names: config.queues.factories.queue_names(),
        queue_factory_roles: config
            .queues
            .factories
            .clone()
            .with_project_wait_scopes(&config),
        ..Default::default()
    }
}

#[test]
fn queue_memmem_scope_preserves_factory_reexport_facts() {
    let root = fixture("queue-ast-hop/factory-reexports/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    assert!(
        matches!(scoped.queue_paths, FamilyPathScope::Only(_)),
        "factory-reexports uses relative imports, so memmem seeds plus closure should narrow"
    );
    assert_eq!(
        queue_report(&root, collect_check_facts(&root, files.clone(), scoped)),
        queue_report(&root, collect_check_facts(&root, files, queue_plan(&root))),
    );
}

fn queue_report(root: &Path, facts: crate::codebase::check_facts::CheckFactMap) -> String {
    let report = crate::queue::analyze_project_with_facts(root, None, &[], &facts).unwrap();
    format!(
        "{:#?}\n{:#?}\n{:#?}",
        report.check, report.edges, report.diagnostics
    )
}

#[test]
fn unique_exports_include_scope_preserves_fixture_findings() {
    let root = fixture("config-v2/rule-path-filters/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("unique-exports fixture config");
    let mut scoped = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let unrestricted = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    let scoped_facts = collect_check_facts(&root, files.clone(), scoped);
    let full_facts = collect_check_facts(&root, files, unrestricted);
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &scoped_facts,
    )
    .unwrap();
    let full_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &full_facts,
    )
    .unwrap();
    assert_eq!(scoped_findings, full_findings);
}

#[test]
fn unique_exports_project_scope_keeps_reexport_origin_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-unique-canonical");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("unique-exports suppression fixture config");
    let mut scoped = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let origin =
        crate::codebase::ts_resolver::normalize_path(&root.join("shared/collision-origin.ts"));
    assert!(
        crate::codebase::check_facts::family_scope::applies(&scoped.symbols_paths, &origin),
        "reexport origins outside projects must stay in the symbols closure"
    );
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &collect_check_facts(&root, files.clone(), scoped),
    )
    .expect("scoped unique-exports should keep reexport origin facts");
    let full_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &collect_check_facts(
            &root,
            files,
            CheckFactPlan {
                symbols: true,
                source: true,
                ..Default::default()
            },
        ),
    )
    .unwrap();
    assert_eq!(scoped_findings, full_findings);
}
