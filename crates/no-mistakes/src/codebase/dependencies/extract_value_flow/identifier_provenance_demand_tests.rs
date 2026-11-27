use super::*;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/identifier-provenance-demand.mts",
    )
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
fn captured_identifier_write_scan_requires_identifier_provenance() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/identifier-guard-write-demand.mts",
    );
    for identifier_provenance in [false, true] {
        let facts = collect_ts_facts(
            std::slice::from_ref(&path),
            TsFactPlan {
                value_flow: true,
                identifier_provenance,
                ..Default::default()
            },
        );
        let facts = &facts[&path];
        assert!(facts.parse_error.is_none());
        let flow = &facts.value_flow;
        assert_eq!(flow.identifier_writes.len(), 2);
        assert!(flow
            .identifier_writes
            .iter()
            .all(|write| write.context.is_some() == identifier_provenance));
        let guarded_names = flow
            .finite_identifier_guards
            .iter()
            .filter(|guard| matches!(&guard.kind, GuardKind::ClosedSet { collection, .. } if collection == "allowed"))
            .map(|guard| guard.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            guarded_names,
            if identifier_provenance {
                vec!["local"]
            } else {
                vec!["local", "captured"]
            },
        );
    }
}
