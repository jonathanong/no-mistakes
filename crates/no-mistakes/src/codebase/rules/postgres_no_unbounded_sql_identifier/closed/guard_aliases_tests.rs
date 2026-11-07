use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::{ValueBinding, ValueCall};
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn guard_aliases_reject_cycles_mutable_bindings_and_unknown_initializers() {
    let file = PathBuf::from("query.ts");
    let mut ts = TsFileFacts::default();
    ts.value_flow.calls.push(ValueCall {
        offset: 100,
        end: 110,
        source_callee: "read".into(),
        context: None,
        callee: ValueExpression::Unknown,
        arguments: Arc::new(Vec::new()),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    });
    for (index, (name, next)) in [
        ("first", Some("second")),
        ("second", Some("first")),
        ("mutable", Some("first")),
        ("opaque", None),
    ]
    .into_iter()
    .enumerate()
    {
        ts.value_flow.bindings.push(ValueBinding {
            immutable: name != "mutable",
            owner: None,
            name: name.into(),
            scope: 1,
            declared_at: index as u32 + 1,
            repeated: false,
            invalidated_at: None,
            mutation_contexts: Default::default(),
            value: next.map_or(ValueExpression::Unknown, |name| ValueExpression::Named {
                name: name.into(),
                scope: Some(1),
                offset: 50,
                context: None,
            }),
        });
    }
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let bindings = Bindings::new(&facts);
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (100, 110),
        call: &embedded,
    };
    for (name, members) in [
        ("first", Vec::new()),
        ("mutable", Vec::new()),
        ("opaque", Vec::new()),
        ("missing", Vec::new()),
        ("first", vec!["table".into()]),
    ] {
        assert!(!proven(
            &graph,
            &opts,
            &facts,
            &bindings,
            &call,
            &facts.get_ts_facts(&file).unwrap().value_flow,
            (name, 1, 100, &members),
        ));
    }
}
