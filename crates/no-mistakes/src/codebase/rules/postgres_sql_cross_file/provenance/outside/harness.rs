use super::super::super::SqlCall;
use super::super::{identifier_sources, Bindings, Provenance};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, ValueBinding, ValueCall, ValueExpression, ValueFlowFacts,
    ValueParameter,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::fx_set;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: Some(0),
        offset: 4,
        context: None,
    }
}

pub(super) fn reference(name: &str) -> Provenance {
    Provenance::Reference {
        name: name.into(),
        scope: 0,
        offset: 4,
        members: Vec::new(),
    }
}

pub(super) fn binding(name: &str, value: ValueExpression) -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: name.to_string(),
        scope: 0,
        declared_at: 1,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        value,
    }
}

pub(super) fn parameter(name: &str) -> ValueParameter {
    ValueParameter {
        owner: CallableId(1),
        index: 0,
        name: name.to_string(),
        scope: 0,
        default: ValueExpression::Undefined,
        projection: Vec::new(),
        initialization_order: 0,
        default_order: 0,
        default_mutations: None,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        type_reference: None,
    }
}

pub(super) fn process_env(key: &str) -> ValueExpression {
    let root = ValueExpression::Named {
        name: "process".to_string(),
        scope: None,
        offset: 0,
        context: None,
    };
    ValueExpression::Member(
        Arc::new(ValueExpression::Member(Arc::new(root), "env".into())),
        key.into(),
    )
}

pub(super) fn sources(
    bindings: Vec<ValueBinding>,
    outside: ValueExpression,
    imports: Vec<ImportedBinding>,
    spans: &[(u32, u32)],
) -> std::collections::BTreeSet<Provenance> {
    let file = PathBuf::from("query.ts");
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        Arc::new(CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: imports,
                value_flow: ValueFlowFacts {
                    parameters: vec![parameter("table"), parameter("uid"), parameter("free")],
                    bindings,
                    calls: vec![call(outside)],
                    ..ValueFlowFacts::default()
                },
                ..TsFileFacts::default()
            }),
            ..CheckFileFacts::default()
        }),
    );
    let sql = EmbeddedSqlCall {
        sql_text: Some("SELECT id FROM sql_placeholder_1 WHERE id = sql_placeholder_2".into()),
        ..EmbeddedSqlCall::default()
    };
    let calls: Vec<_> = std::iter::once((10, 20))
        .chain(spans.iter().copied())
        .map(|span| SqlCall {
            file: file.clone(),
            span,
            call: &sql,
        })
        .collect();
    let found = Bindings::with_calls(&facts, &calls);
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    identifier_sources(
        &graph,
        &facts,
        &SqlCall {
            file,
            span: (10, 20),
            call: &sql,
        },
        &found,
    )
}

pub(super) fn call(outside: ValueExpression) -> ValueCall {
    ValueCall {
        offset: 10,
        end: 20,
        source_callee: String::new(),
        context: None,
        callee: ValueExpression::Unknown,
        arguments: Arc::new(vec![ValueExpression::Derived(Arc::from([
            named("table"),
            outside,
        ]))]),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    }
}

pub(super) fn expect(
    bindings: Vec<ValueBinding>,
    outside: ValueExpression,
    imports: Vec<ImportedBinding>,
    spans: &[(u32, u32)],
    extra: Vec<Provenance>,
) {
    let mut expected = BTreeSet::from([reference("table")]);
    expected.extend(extra);
    assert_eq!(sources(bindings, outside, imports, spans), expected);
}
