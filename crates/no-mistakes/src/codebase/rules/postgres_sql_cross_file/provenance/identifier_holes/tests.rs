use super::super::super::SqlCall;
use super::super::bindings::Bindings;
use super::split;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{ValueCall, ValueExpression, ValueFlowFacts};
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn selected<'a>(
    facts: &'a CheckFactMap,
    call: &SqlCall<'a>,
    bindings: &Bindings<'a>,
) -> Option<Vec<&'a ValueExpression>> {
    split(facts, call, bindings).map(|(holes, _)| holes)
}

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.to_string(),
        scope: Some(0),
        offset: 4,
        context: None,
    }
}

fn call(arguments: Vec<ValueExpression>) -> ValueCall {
    ValueCall {
        offset: 10,
        end: 20,
        source_callee: String::new(),
        context: None,
        callee: ValueExpression::Unknown,
        arguments: Arc::new(arguments),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    }
}

fn harness(flow: ValueFlowFacts) -> (CheckFactMap, PathBuf, EmbeddedSqlCall) {
    let file = PathBuf::from("query.ts");
    let ts = TsFileFacts {
        value_flow: flow,
        ..TsFileFacts::default()
    };
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        Arc::new(CheckFileFacts {
            ts: Arc::new(ts),
            ..CheckFileFacts::default()
        }),
    );
    let sql = EmbeddedSqlCall {
        sql_text: Some("SELECT sql_placeholder_1 FROM sql_placeholder_2".to_string()),
        ..EmbeddedSqlCall::default()
    };
    (facts, file, sql)
}

#[test]
fn selected_keeps_identifier_holes_and_drops_misaligned_templates() {
    let derived = ValueExpression::Derived(Arc::from([named("column"), named("table")]));
    let (facts, file, sql) = harness(ValueFlowFacts {
        calls: vec![call(vec![derived])],
        ..ValueFlowFacts::default()
    });
    let bindings = Bindings::new(&facts);
    let found = selected(
        &facts,
        &super::super::super::SqlCall {
            file: file.clone(),
            span: (10, 20),
            call: &sql,
        },
        &bindings,
    )
    .expect("aligned holes");
    assert_eq!(found.len(), 2);

    let (facts, file, sql) = harness(ValueFlowFacts {
        bindings: vec![crate::codebase::dependencies::extract::ValueBinding {
            immutable: true,
            owner: None,
            name: "sql".to_string(),
            scope: 0,
            declared_at: 1,
            repeated: false,
            invalidated_at: None,
            mutation_contexts: crate::fx::fx_set(),
            value: ValueExpression::Awaited(Arc::new(ValueExpression::Derived(Arc::from([
                named("column"),
                named("table"),
            ])))),
        }],
        calls: vec![call(vec![named("sql")])],
        ..ValueFlowFacts::default()
    });
    let bindings = Bindings::new(&facts);
    assert!(selected(
        &facts,
        &super::super::super::SqlCall {
            file: file.clone(),
            span: (10, 20),
            call: &sql,
        },
        &bindings,
    )
    .is_some());

    let (facts, file, sql) = harness(ValueFlowFacts {
        calls: vec![call(vec![
            ValueExpression::Unknown,
            named("column"),
            named("table"),
        ])],
        ..ValueFlowFacts::default()
    });
    let bindings = Bindings::new(&facts);
    assert!(selected(
        &facts,
        &super::super::super::SqlCall {
            file,
            span: (10, 20),
            call: &sql,
        },
        &bindings,
    )
    .is_some());

    let (facts, file, sql) = harness(ValueFlowFacts {
        calls: vec![call(vec![ValueExpression::Derived(Arc::from([named(
            "only",
        )]))])],
        ..ValueFlowFacts::default()
    });
    let bindings = Bindings::new(&facts);
    assert!(selected(
        &facts,
        &super::super::super::SqlCall {
            file: file.clone(),
            span: (10, 20),
            call: &sql,
        },
        &bindings,
    )
    .is_none());

    let (facts, file, sql) = harness(ValueFlowFacts {
        bindings: vec![crate::codebase::dependencies::extract::ValueBinding {
            immutable: true,
            owner: None,
            name: "sql".to_string(),
            scope: 0,
            declared_at: 1,
            repeated: false,
            invalidated_at: None,
            mutation_contexts: crate::fx::fx_set(),
            value: named("sql"),
        }],
        calls: vec![call(vec![named("sql")])],
        ..ValueFlowFacts::default()
    });
    let bindings = Bindings::new(&facts);
    assert!(selected(
        &facts,
        &super::super::super::SqlCall {
            file,
            span: (99, 100),
            call: &sql,
        },
        &bindings,
    )
    .is_none());
    let bindings = Bindings::new(&facts);
    assert!(selected(
        &facts,
        &super::super::super::SqlCall {
            file: PathBuf::from("query.ts"),
            span: (10, 20),
            call: &sql,
        },
        &bindings,
    )
    .is_none());
}
