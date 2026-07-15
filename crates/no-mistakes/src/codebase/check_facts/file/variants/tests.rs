use super::{collect_file_fact_variants_with_session, CheckFactVariant};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::rules::server_route_client_boundary::FileFacts;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn shared_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/shared-facts/fixture")
        .join(name)
}

fn ast_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-source/fixture/facts")
        .join(name)
}

fn plans() -> [CheckFactPlan; 2] {
    [
        CheckFactPlan {
            source: true,
            ..CheckFactPlan::default()
        },
        CheckFactPlan {
            source: true,
            server_route_client_boundary: true,
            ..CheckFactPlan::default()
        },
    ]
}

fn collect(root: &Path, path: &Path) -> Vec<Option<super::super::CheckFileFacts>> {
    let plans = plans();
    let variants = plans
        .iter()
        .map(|plan| CheckFactVariant {
            root,
            plan,
            playwright: None,
        })
        .collect::<Vec<_>>();
    collect_file_fact_variants_with_session(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        path,
        &variants,
    )
}

fn assert_boundary_is_variant_specific(facts: &[Option<super::super::CheckFileFacts>]) {
    assert_eq!(facts.len(), 2);
    assert_eq!(
        facts[0].as_ref().unwrap().server_route_client_boundary,
        None
    );
    assert_eq!(
        facts[1].as_ref().unwrap().server_route_client_boundary,
        Some(FileFacts::default())
    );
}

#[test]
fn batch_read_errors_preserve_requested_boundary_facts() {
    let root = shared_fixture("");
    let path = shared_fixture("src/unreadable.ts");
    let facts = collect(&root, &path);

    assert!(facts
        .iter()
        .all(|facts| facts.as_ref().unwrap().parse_error.is_some()));
    assert_boundary_is_variant_specific(&facts);
}

#[test]
fn batch_recovered_diagnostics_preserve_requested_boundary_facts() {
    let root = shared_fixture("");
    let path = shared_fixture("src/invalid.ts");
    let facts = collect(&root, &path);

    assert!(facts.iter().all(|facts| facts.as_ref().unwrap().parsed));
    assert!(facts
        .iter()
        .all(|facts| facts.as_ref().unwrap().parse_error.is_some()));
    assert_boundary_is_variant_specific(&facts);
}

#[test]
fn batch_unsupported_sources_preserve_requested_boundary_facts() {
    let root = ast_fixture("");
    let path = ast_fixture("unknown-extension.source");
    let facts = collect(&root, &path);

    assert!(facts.iter().all(|facts| facts
        .as_ref()
        .unwrap()
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("unsupported file type"))));
    assert_boundary_is_variant_specific(&facts);
}

#[test]
fn batch_legacy_failures_preserve_requested_boundary_facts() {
    let root = ast_fixture("");
    let path = ast_fixture("unknown-extension.source");
    let source = Arc::<str>::from(std::fs::read_to_string(&path).unwrap());
    let plans = plans();
    let variants = plans
        .iter()
        .map(|plan| CheckFactVariant {
            root: &root,
            plan,
            playwright: None,
        })
        .collect::<Vec<_>>();
    let indexed = variants.iter().enumerate().collect::<Vec<_>>();
    let mut facts = vec![None, None];

    super::errors::fill_parse_errors(
        &mut facts,
        indexed,
        &path,
        &source,
        true,
        anyhow::anyhow!("legacy parser panic"),
    );

    assert!(
        facts
            .iter()
            .all(|facts| facts.as_ref().unwrap().parse_error.as_deref()
                == Some("legacy parser panic"))
    );
    assert_boundary_is_variant_specific(&facts);
}
