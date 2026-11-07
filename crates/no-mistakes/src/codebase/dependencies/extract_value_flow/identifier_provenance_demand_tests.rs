use super::*;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/identifier-provenance-demand.mts",
    )
}

#[test]
fn ordinary_value_flow_keeps_scalar_assignment_indexes_cold() {
    let path = fixture();
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    for identifier_provenance in [false, true] {
        let mut collector = ImportCollector {
            collect_value_flow: true,
            collect_identifier_provenance: identifier_provenance,
            local_stack: vec![fx_set()],
            type_local_stack: vec![fx_set()],
            lexical_scope_ids: vec![0],
            next_lexical_scope_id: 1,
            ..Default::default()
        };
        collector.lexical_scope_parents.insert(0, None);
        crate::codebase::dependencies::extract::predeclare_program_value_bindings(
            &mut collector,
            &parsed.program,
        );
        collector.visit_program(&parsed.program);
        assert_eq!(
            collector
                .identifier_initializers
                .values()
                .map(|names| names.len())
                .sum::<usize>(),
            if identifier_provenance { 2 } else { 0 },
        );
        assert_eq!(
            collector
                .value_flow
                .identifier_writes
                .iter()
                .filter(|write| write.assigned.is_some())
                .count(),
            usize::from(identifier_provenance),
        );
        assert!(!collector.value_flow.bindings.is_empty());
    }
}

#[test]
fn prepared_call_facts_collect_scalar_provenance_only_on_demand() {
    let path = fixture();
    let ordinary = GraphBuildPlan {
        calls: true,
        ..Default::default()
    };
    let requested = GraphBuildPlan {
        identifier_provenance: true,
        ..ordinary
    };
    let plain_plan = ordinary.ts_fact_plan();
    let requested_plan = requested.ts_fact_plan();
    assert!(!plain_plan.identifier_provenance);
    assert!(requested_plan.identifier_provenance);
    assert!(!plain_plan.covers(requested_plan));
    let mut merged = ordinary;
    merged.include(requested);
    assert!(merged.identifier_provenance);
    let mut merged_facts = plain_plan;
    merged_facts.include(requested_plan);
    assert!(merged_facts.covers(requested_plan));
    let facts = |plan| collect_ts_facts(std::slice::from_ref(&path), plan);
    let ordinary_facts = facts(plain_plan);
    let requested_facts = facts(requested_plan);
    let plain = &ordinary_facts[&path];
    let selected = &requested_facts[&path];
    assert!(plain.parse_error.is_none() && selected.parse_error.is_none());
    assert!(plain.value_flow.identifier_text_origins.is_none());
    assert_eq!(plain.imports, selected.imports);
    assert_eq!(plain.function_calls, selected.function_calls);
    assert_eq!(plain.value_flow.bindings, selected.value_flow.bindings);
    assert!(plain
        .value_flow
        .identifier_writes
        .iter()
        .all(|write| write.assigned.is_none() && write.context.is_none()));
    let assigned = selected
        .value_flow
        .identifier_writes
        .iter()
        .filter_map(|write| write.assigned.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(assigned.len(), 1);
    assert!(matches!(assigned[0].value, ValueExpression::Derived(_)));
    assert!(assigned[0].initial.is_some());
    let demand_only = TsFactPlan {
        identifier_provenance: true,
        ..Default::default()
    };
    assert!(!demand_only.is_empty());
    assert!(demand_only.needs_value_flow() && demand_only.needs_import_facts());
    assert_eq!(
        facts(demand_only)[&path].value_flow.identifier_writes,
        selected.value_flow.identifier_writes
    );
}

#[test]
fn direct_value_flow_extraction_retains_scalar_assignment_compatibility() {
    let path = fixture();
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(
        &parsed.program, &source, false, false, true,
    );
    assert_eq!(
        facts
            .value_flow
            .identifier_writes
            .iter()
            .filter(|write| write.assigned.is_some())
            .count(),
        1
    );
}
