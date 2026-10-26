use super::{
    applies, configure_family_path_scopes, fixture, unique_exports_plan,
    unique_exports_scoped_and_full,
};
use crate::codebase::check_facts::collect_check_facts;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;
use std::path::PathBuf;

#[test]
fn unique_exports_include_scope_preserves_fixture_findings() {
    let root = fixture("config-v2/rule-path-filters/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("unique-exports fixture config");
    let mut scoped = unique_exports_plan();
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let unrestricted = unique_exports_plan();
    let scoped_facts = collect_check_facts(&root, files.clone(), scoped);
    let full_facts = collect_check_facts(&root, files, unrestricted);
    let yml = root.join(".no-mistakes.yml");
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&yml),
        None,
        &scoped_facts,
    )
    .unwrap();
    let full_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&yml),
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
    let mut scoped = unique_exports_plan();
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let origin = normalize_path(&root.join("shared/collision-origin.ts"));
    assert!(
        applies(&scoped.symbols_paths, &origin),
        "reexport origins outside projects must stay in the symbols closure"
    );
    let yml = root.join(".no-mistakes.yml");
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&yml),
        None,
        &collect_check_facts(&root, files.clone(), scoped),
    )
    .expect("scoped unique-exports should keep reexport origin facts");
    let full_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&yml),
        None,
        &collect_check_facts(&root, files, unique_exports_plan()),
    )
    .unwrap();
    assert_eq!(scoped_findings, full_findings);
}

#[test]
fn unique_exports_workspace_alias_preserves_fixture_findings() {
    let root = fixture("codebase-analysis/unique-exports-project-scope-workspace/fixture");
    let (scoped, full) = unique_exports_scoped_and_full(&root);
    let origin = normalize_path(&root.join("packages/source/src/index.ts"));
    assert!(
        applies(&scoped.0, &origin),
        "workspace package reexport origins must keep symbol facts"
    );
    assert_eq!(scoped.1, full);
}
