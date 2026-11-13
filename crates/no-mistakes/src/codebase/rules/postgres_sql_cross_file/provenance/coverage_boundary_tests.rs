use super::{classify, interpolations, text_tag, value_call_at, Bindings, SqlCall};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::postgres::EmbeddedSqlCall;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn missing_facts_and_unmatched_calls_add_no_identifier_provenance() {
    let file = PathBuf::from("query.ts");
    let facts = CheckFactMap::default();
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let bindings = Bindings::new(&facts);
    let sql = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (1, 2),
        call: &sql,
    };
    assert!(value_call_at(&facts, &file, 1, 2).is_none());
    assert!(interpolations(&graph, &facts, &call, &bindings).is_empty());
    assert!(text_tag::projection_span(
        &bindings,
        &file,
        &ValueExpression::Member(Arc::new(ValueExpression::Call(1, 2)), "text".into()),
    )
    .is_none());
    let mut out = BTreeSet::new();
    classify(
        &graph,
        &facts,
        &file,
        &ValueExpression::Call(1, 2),
        &mut out,
        &bindings,
    );
    assert!(out.is_empty());
}
