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

    assert!(facts.iter().all(|facts| {
        facts
            .as_ref()
            .unwrap()
            .parse_error
            .as_deref()
            .is_some_and(|error| error.contains("unsupported file type"))
    }));
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

#[test]
fn unknown_extension_production_import_does_not_retain_unselected_raw_source() {
    use crate::codebase::check_facts::FamilyPathScope;
    let root = ast_fixture("");
    let path = ast_fixture("unknown-extension.source");
    let plan = CheckFactPlan {
        production_import_paths: Arc::new([path.clone()].into_iter().collect()),
        raw_source: true,
        raw_source_paths: FamilyPathScope::Only(Arc::new(Default::default())),
        ..Default::default()
    };
    let facts = collect_file_fact_variants_with_session(
        &crate::codebase::analysis_session::AnalysisSession::disabled(),
        &path,
        &[CheckFactVariant {
            root: &root,
            plan: &plan,
            playwright: None,
        }],
    );
    let facts = facts[0].as_ref().unwrap();
    assert!(facts.production_imports.is_some());
    assert!(facts.source.is_none());
}

#[test]
fn recovered_legacy_variant_keeps_symbols_and_source() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/importers/malformed-runner-helper");
    let path = root.join("setup.ts");
    let source = Arc::<str>::from(std::fs::read_to_string(&path).unwrap());
    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::from_path(&path).unwrap();
    let parsed = oxc_parser::Parser::new(&allocator, &source, source_type).parse();
    let plan = CheckFactPlan {
        symbols: true,
        source: true,
        legacy_symbol_paths: [path.clone()].into_iter().collect(),
        ..Default::default()
    };
    let facts = super::errors::recovered_error_facts(
        &path,
        &CheckFactVariant {
            root: &root,
            plan: &plan,
            playwright: None,
        },
        &source,
        &parsed.program,
        &source,
        "fixture parser diagnostic".into(),
        super::VariantRecovery {
            symbols: true,
            fatal_parse_error: false,
        },
    );
    assert!(facts.symbols.is_some());
    assert!(facts.legacy_symbols.is_some());
    assert!(facts.source.is_some());
    assert_eq!(
        facts.parse_error.as_deref(),
        Some("fixture parser diagnostic")
    );
}
