use super::{unique_closed_collection, GuardKind, RegistryKind, SetMethod};
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/codebase-analysis/closed-sql-identifier-domains/facts.ts"
));

#[test]
fn closed_domain_facts_reject_open_shapes() {
    let flow = parse_flow(SOURCE, true);
    let mut names: Vec<_> = flow
        .closed_string_registries
        .iter()
        .map(|registry| registry.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "ALLOWED",
            "KEEP_ARRAY",
            "KEEP_FROZEN",
            "KEEP_FROZEN_MAP",
            "KEEP_MAP",
            "KEEP_OBJECT",
            "KEEP_SET",
            "KEEP_TEMPLATE",
            "TABLES",
            "TABLES",
        ]
    );
    assert!(unique_closed_collection(
        &flow,
        "KEEP_SET",
        RegistryKind::Set
    ));
    assert!(unique_closed_collection(
        &flow,
        "KEEP_OBJECT",
        RegistryKind::Object
    ));
    assert!(unique_closed_collection(
        &flow,
        "KEEP_ARRAY",
        RegistryKind::Array
    ));
    assert!(unique_closed_collection(
        &flow,
        "KEEP_MAP",
        RegistryKind::Map
    ));
    assert!(!unique_closed_collection(
        &flow,
        "KEEP_SET",
        RegistryKind::Array
    ));
    assert!(!unique_closed_collection(
        &flow,
        "ALLOWED",
        RegistryKind::Set
    ));
    assert!(!unique_closed_collection(
        &flow,
        "TABLES",
        RegistryKind::Map
    ));
    assert!(!unique_closed_collection(
        &flow,
        "missing",
        RegistryKind::Set
    ));
    let mut bounded: Vec<_> = flow
        .bounded_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect();
    bounded.sort_unstable();
    assert_eq!(
        bounded,
        ["copied", "copiedStatic", "fromArray", "fromMap", "goodName"]
    );
    assert_eq!(flow.finite_identifier_guards.len(), 6);
    assert!(flow.finite_identifier_guards.iter().any(|guard| {
        guard.name == "ALLOWED"
            && matches!(
                guard.kind,
                GuardKind::ClosedSet {
                    ref collection,
                    method: SetMethod::Has,
                } if collection == "ALLOWED"
            )
    }));
    assert!(flow.finite_identifier_guards.iter().any(|guard| {
        matches!(
            guard.kind,
            GuardKind::ClosedSet {
                ref collection,
                method: SetMethod::Includes,
            } if collection == "KEEP_ARRAY"
        )
    }));
    assert!(flow.finite_identifier_guards.iter().any(|guard| {
        matches!(
            &guard.kind,
            GuardKind::Assertion { callee, argument: 0 } if callee == "assertWhitelistedSqlIdentifier"
        )
    }));
    let skipped = parse_flow(SOURCE, false);
    assert!(skipped.closed_string_registries.is_empty());
    assert!(skipped.bounded_bindings.is_empty());
    assert!(skipped.finite_identifier_guards.is_empty());
}

fn parse_flow(
    source: &str,
    collect_value_flow: bool,
) -> crate::codebase::dependencies::extract::ValueFlowFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        collect_value_flow,
    )
    .value_flow
}
