use super::super::{Bindings, Provenance};
use super::interpolations;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::postgres::{EmbeddedSqlCall, TrustedSqlTag};
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn enclosing_parameterized_tags_do_not_hide_nested_executor_sources() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-fragments");
    let file = root.join("src/negative.ts");
    let collected = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(collected.get(&file).unwrap().clone()),
            ..Default::default()
        }
        .into(),
    );
    let tags = [TrustedSqlTag {
        module: "@example/sql".into(),
        name: "sql".into(),
    }];
    let bindings = Bindings::new(&facts).with_trusted_tags(&tags);
    let graph = from_raw_maps(root, Default::default(), Default::default());
    let mut queries: Vec<_> = collected
        .get(&file)
        .unwrap()
        .value_flow
        .calls
        .iter()
        .filter(|call| {
            call.source_callee == "query"
                && matches!(call.arguments.first(), Some(ValueExpression::Derived(parts)) if parts.len() == 2)
        })
        .collect();
    queries.sort_by_key(|call| call.offset);
    assert_eq!(queries.len(), 2);
    let embedded = EmbeddedSqlCall::default();
    for (query, expected) in queries
        .into_iter()
        .zip([["FIRST", "SECOND"], ["THIRD", "FOURTH"]])
    {
        let call = SqlCall {
            file: file.clone(),
            span: (query.offset, query.end),
            call: &embedded,
        };
        assert_eq!(
            interpolations(&graph, &facts, &call, &bindings),
            BTreeSet::from(expected.map(|key| Provenance::ProcessEnv(key.into()))),
            "nested executor at {}",
            query.offset,
        );
    }
}
