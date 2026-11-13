use super::flow::Seen;
use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{ValueBinding, ValueExpression, ValueFlowFacts};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn binding(name: &str, scope: usize, declared_at: u32, value: ValueExpression) -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: name.into(),
        scope,
        declared_at,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: crate::fx::fx_set(),
        value,
    }
}

fn facts_for(flow: ValueFlowFacts) -> (CheckFactMap, PathBuf) {
    let file = PathBuf::from("query.ts");
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                value_flow: flow,
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    (facts, file)
}

#[test]
fn repeated_bindings_sort_by_declaration_and_ignore_later_declarations() {
    let (facts, file) = facts_for(ValueFlowFacts {
        bindings: vec![
            binding("entry", 1, 8, ValueExpression::Unknown),
            binding("entry", 1, 2, ValueExpression::Unknown),
        ],
        ..Default::default()
    });
    let bindings = Bindings::new(&facts);
    let visible = bindings
        .value(&file, 1, "entry", 5)
        .expect("earlier binding");
    assert_eq!(visible.declared_at, 2);
    assert!(bindings.value(&file, 1, "entry", 1).is_none());
}

#[test]
fn repeated_seen_assignment_is_not_an_open_identifier() {
    let file = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("3304-registry-routes/src/assignments.ts"),
    );
    let source = std::fs::read_to_string(&file).unwrap();
    let plan = crate::codebase::ts_source::facts::TsFactPlan {
        value_flow: true,
        identifier_provenance: true,
        ..crate::codebase::ts_source::facts::TsFactPlan::imports()
    };
    let ts = crate::codebase::ts_source::facts::collect_ts_facts(std::slice::from_ref(&file), plan)
        .remove(&file)
        .unwrap();
    assert!(ts.parse_error.is_none(), "{:?}", ts.parse_error);
    let mut facts = CheckFactMap {
        files: vec![file.clone()],
        ..Default::default()
    };
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts).with_numeric_parameter_bounds();
    let call = facts
        .get_ts_facts(&file)
        .unwrap()
        .value_flow
        .calls
        .iter()
        .find(|call| {
            call.source_callee == "probe"
                && source[call.offset as usize..]
                    .lines()
                    .next()
                    .is_some_and(|line| line.contains("probe(first)"))
        })
        .expect("probe(first)");
    let ValueExpression::Named { name, scope, .. } = &call.arguments[0] else {
        panic!("probe reads a named binding");
    };
    let scope = scope.unwrap_or(0);
    let completed_at = facts
        .get_ts_facts(&file)
        .unwrap()
        .value_flow
        .identifier_writes
        .iter()
        .filter(|write| write.name == *name && write.scope == scope)
        .filter_map(|write| write.assigned.as_deref())
        .map(|assigned| assigned.completed_at)
        .filter(|completed_at| *completed_at <= call.end)
        .max()
        .unwrap_or_else(|| panic!("missing assignment for {name} in scope {scope}"));
    let mut seen = Seen {
        bindings: crate::fx::fx_set(),
        calls: crate::fx::fx_set(),
    };
    seen.bindings.insert((scope, name.clone(), completed_at));
    assert_eq!(
        bindings.assignment_unbounded(&file, scope, name, call.end, &mut seen),
        Some(false)
    );
}

#[test]
fn row_projection_drops_invalidated_and_not_yet_declared_iterations() {
    let (facts, file) = facts_for(ValueFlowFacts {
        bindings: vec![ValueBinding {
            invalidated_at: Some(30),
            value: ValueExpression::Call(10, 20),
            ..binding("visible", 1, 4, ValueExpression::Unknown)
        }],
        iteration_bindings: vec![
            binding("early", 1, 1, ValueExpression::Call(10, 20)),
            binding("late", 1, 40, ValueExpression::Call(10, 20)),
        ],
        ..Default::default()
    });
    let sql = crate::codebase::postgres::EmbeddedSqlCall::default();
    let call = crate::codebase::rules::postgres_sql_cross_file::SqlCall {
        file: file.clone(),
        span: (10, 20),
        call: &sql,
    };
    let calls = [call];
    let bindings = Bindings::with_calls(&facts, &calls);
    assert!(bindings.query_projection(&file, 1, "visible", 8));
    assert!(!bindings.query_projection(&file, 1, "visible", 30));
    assert!(bindings.query_projection(&file, 1, "early", 8));
    assert!(!bindings.query_projection(&file, 1, "late", 8));
    let missing = PathBuf::from("missing.ts");
    assert!(!bindings.parameter(&missing, 1, "input"));
    assert!(!bindings.closed_parameter(&missing, 1, "input"));
}
