use super::super::load_gap::missing_static_load_gap;
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands;
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::sync::Arc;

const LOADS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/loads.mts"
));
const DYNAMIC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/dynamic.mts"
));

fn extracted(source: &str) -> TsFileFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source,
        SourceType::from_path("created.mts").expect("mts source type"),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:#?}",
        parsed.diagnostics
    );
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        true,
    );
    TsFileFacts {
        source: Some(Arc::<str>::from(source)),
        imports: facts.imports,
        namespaces: facts.namespaces,
        value_flow: facts.value_flow,
        ..TsFileFacts::default()
    }
}

#[test]
fn a_literal_created_require_is_not_a_static_load_gap() {
    let ts = extracted(LOADS);
    assert_eq!(
        missing_static_load_gap(&ts, LOADS, "loads.mts"),
        None,
        "{:?}",
        ts.namespaces.literal_import_spans
    );
}

#[test]
fn a_dynamic_created_require_stays_a_static_load_gap() {
    let ts = extracted(DYNAMIC);
    let gap = missing_static_load_gap(&ts, DYNAMIC, "dynamic.mts");
    assert!(
        gap.as_deref()
            .is_some_and(|message| message.contains("no recorded specifier")),
        "{gap:?}"
    );
}
