use super::*;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn collector() -> ImportCollector {
    let file = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-guards/src/metadata-row-helpers.ts");
    let prepared = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            value_flow: true,
            ..TsFactPlan::imports()
        },
    );
    let ts = prepared.get(&file).unwrap();
    assert!(ts.parse_error.is_none());
    let mut collector = ImportCollector {
        value_flow: ts.value_flow.clone(),
        ..Default::default()
    };
    for binding in collector
        .value_flow
        .bindings
        .iter()
        .filter(|binding| binding.name == "selected")
    {
        collector
            .metadata_rows
            .named
            .entry(binding.scope)
            .or_default()
            .insert(
                binding.name.clone(),
                super::super::ReadUse {
                    fields: ["key", "schema", "table"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    invalid: false,
                },
            );
    }
    collector
}

#[test]
fn metadata_proof_boundaries_ignore_incomplete_lookups_and_invalidate_only_in_range_writes() {
    let mut collector = collector();
    let mut incomplete = collector
        .value_flow
        .bounded_bindings
        .iter()
        .find(|bounded| bounded.requires_guard)
        .unwrap()
        .clone();
    incomplete.name = "incomplete".into();
    incomplete.members.clear();
    incomplete.source = None;
    collector.value_flow.bounded_bindings.push(incomplete);
    let guard = collector
        .value_flow
        .finite_identifier_guards
        .iter()
        .find(|guard| matches!(guard.kind, GuardKind::Lookup))
        .unwrap()
        .clone();
    let GuardSubject::Binding { scope, declared_at } = guard.subject.unwrap() else {
        panic!("binding guard")
    };
    let owner = guard.context.unwrap();
    let offset = collector
        .value_flow
        .returns
        .iter()
        .find_map(|(id, value)| match value {
            ValueExpression::Named { name, offset, .. } if *id == owner && name == &guard.name => {
                Some(*offset)
            }
            _ => None,
        })
        .unwrap();
    let identity = (scope, guard.name.as_str(), declared_at);
    let index = Index::new(&collector);
    assert!(!index.lookups.keys().any(|key| key.1 == "incomplete"));
    assert!(index.guarded(identity, owner, offset));
    for (at, expected) in [(guard.from, true), (guard.from + 1, false), (offset, true)] {
        collector.value_flow.identifier_writes = vec![IdentifierWrite {
            context: None,
            assigned: None,
            scope,
            name: guard.name.clone(),
            members: vec![Some("table".into())],
            at,
        }];
        assert_eq!(
            Index::new(&collector).guarded(identity, owner, offset),
            expected,
            "{at}"
        );
    }
}

#[test]
fn metadata_proof_boundaries_reject_ambiguous_helper_names_and_call_results() {
    let collector = collector();
    let helper = &collector.value_flow.metadata_row_helpers[0];
    let mut index = Index::new(&collector);
    index.names.insert(helper.owner, vec!["getRowOrThrow"]);
    assert!(super::super::candidates::collect(&index, true)
        .iter()
        .any(|candidate| candidate.owner == helper.owner && candidate.valid));
    index
        .names
        .insert(helper.owner, vec!["getRowOrThrow", "alias"]);
    let candidates = super::super::candidates::collect(&index, true);
    let candidate = candidates
        .iter()
        .find(|candidate| candidate.owner == helper.owner)
        .unwrap();
    assert!(!candidate.valid);

    let call = collector
        .value_flow
        .calls
        .iter()
        .find(|call| {
            call.source_callee == "getRowOrThrow"
                && index.results.contains_key(&(call.offset, call.end))
        })
        .unwrap();
    index.calls.insert(helper.owner, vec![call]);
    let candidate = super::super::candidates::Candidate {
        owner: helper.owner,
        sources: vec![&helper.source],
        fields: helper.fields.iter().map(String::as_str).collect(),
        valid: true,
    };
    assert!(super::super::callers::bindings(&index, &candidate).is_some());
    let results = index.results.get_mut(&(call.offset, call.end)).unwrap();
    let binding = results[0];
    results.push(binding);
    assert!(super::super::callers::bindings(&index, &candidate).is_none());
}

#[test]
fn metadata_proof_boundaries_do_not_collect_row_reads_without_value_flow() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/metadata-row-helpers.ts"));
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let expression = parsed
        .program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::VariableDeclaration(declaration) = statement else {
                return None;
            };
            declaration.declarations[0].init.as_ref()
        })
        .unwrap();
    let mut collector = ImportCollector::default();
    collector.note_metadata_row_expression(expression);
    assert!(collector.metadata_rows.named.is_empty());
    assert!(collector.metadata_rows.direct.is_empty());
}
