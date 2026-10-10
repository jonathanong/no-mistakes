use super::*;
use crate::codebase::postgres::source::*;
fn queries(file: &str) -> Vec<PostgresSqlQuery> {
    let facts = facts(file);
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    facts
        .statements
        .into_iter()
        .filter_map(|s| {
            if let PostgresSqlStatementKind::Select { query } = s.facts {
                Some(query)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn scopes_aliases_and_predicate_contexts_are_separate() {
    let queries = queries("query-scopes.sql");
    let q = &queries[0];
    assert!(q.complete, "{:?}", q.unsupported);
    assert_eq!(
        q.ctes
            .iter()
            .map(|c| (&*c.name.identity, c.referenced, c.used))
            .collect::<Vec<_>>(),
        vec![
            ("unused", false, false),
            ("base", true, true),
            ("used", true, true)
        ]
    );
    let equalities: Vec<_> = q.equalities.iter().filter(|e| e.scope_id == 0).collect();
    assert!(equalities
        .iter()
        .any(|e| e.clause == PostgresSqlQueryClause::Where && e.context.mandatory));
    for predicate in ["or", "not", "case", "boolean"] {
        assert!(equalities.iter().any(|e| !e.context.mandatory
            && match predicate {
                "or" => e.context.under_or,
                "not" => e.context.under_not,
                "case" => e.context.under_case,
                _ => e.context.under_boolean_test,
            }));
    }
    assert!(equalities
        .iter()
        .any(|e| e.join_id == Some(0) && e.context.mandatory));
    assert!(equalities
        .iter()
        .any(|e| e.join_id == Some(1) && !e.context.mandatory));
    assert!(q
        .exists
        .iter()
        .any(|e| e.correlated && !e.negated && e.context.mandatory));
    assert!(q
        .exists
        .iter()
        .any(|e| !e.correlated && e.negated && !e.context.mandatory));
    assert!(q.relations.iter().any(|r| r
        .name
        .as_ref()
        .is_some_and(|n| n.parts[0].identity == "Schéma" && n.parts[0].quoted)));
    for clause in [
        PostgresSqlQueryClause::Projection,
        PostgresSqlQueryClause::OrderBy,
        PostgresSqlQueryClause::Limit,
        PostgresSqlQueryClause::Offset,
    ] {
        assert!(q.scopes.iter().any(|s| s.clause == clause));
    }
}
#[test]
fn set_branches_cte_cycles_and_shadowing_are_explicit() {
    let q = queries("query-scopes.sql");
    assert_eq!(q[1].scopes[0].set_operation.as_deref(), Some("UNION"));
    assert_eq!(
        q[1].joins
            .iter()
            .map(|j| j.constraint.as_str())
            .collect::<Vec<_>>(),
        vec!["using", "natural"]
    );
    let recursive = &q[2];
    assert!(recursive.ctes.iter().all(|c| c.cyclic));
    assert!(recursive.ctes[0].used);
    assert!(!recursive.ctes[1].used);
    assert!(!recursive.ctes[2].used);
    let shadow = &q[3];
    assert_eq!(shadow.ctes.len(), 2);
    assert_ne!(shadow.relations[0].cte_id, shadow.relations[1].cte_id);
}
#[test]
fn relation_visibility_is_not_inferred_from_column_spelling() {
    let q = queries("query-scopes.sql");
    assert_eq!(
        q[4].columns
            .iter()
            .map(|c| &c.resolution)
            .collect::<Vec<_>>(),
        vec![
            &PostgresSqlQueryColumnResolution::Resolved,
            &PostgresSqlQueryColumnResolution::Unknown,
            &PostgresSqlQueryColumnResolution::Unknown,
            &PostgresSqlQueryColumnResolution::Unqualified
        ]
    );
    assert_eq!(
        q[5].columns[0].resolution,
        PostgresSqlQueryColumnResolution::Ambiguous
    );
    let grouped = &q[6];
    assert_eq!(
        grouped.columns.last().unwrap().resolution,
        PostgresSqlQueryColumnResolution::Unknown
    );
    let derived = &q[7];
    let refs: Vec<_> = derived.columns.iter().filter(|c| c.scope_id != 0).collect();
    assert_eq!(
        refs[0].resolution,
        PostgresSqlQueryColumnResolution::Unknown
    );
    assert_eq!(refs[1].relation_scope_id, Some(0));
    assert_eq!(
        q[8].joins.iter().map(|j| &j.kind).collect::<Vec<_>>(),
        vec![
            &PostgresSqlQueryJoinKind::Right,
            &PostgresSqlQueryJoinKind::Full,
            &PostgresSqlQueryJoinKind::Cross
        ]
    );
}
#[test]
fn nested_queries_and_unsupported_syntax_have_observable_facts() {
    let q = queries("query-subqueries.sql");
    assert!(q.iter().all(|q| q.complete), "{q:?}");
    assert!(q[0].scopes.len() >= 24);
    let q = queries("query-unsupported.sql");
    assert!(
        q.iter().all(|q| !q.complete && !q.unsupported.is_empty()),
        "{:?}",
        q.iter()
            .map(|q| (&q.relations, q.complete, &q.unsupported))
            .collect::<Vec<_>>()
    );
}
#[test]
fn query_spans_use_source_utf8_bytes() {
    let source = fixture("query-scopes.sql");
    let q = queries("query-scopes.sql");
    let relation = q[0]
        .relations
        .iter()
        .find(|r| {
            r.name
                .as_ref()
                .is_some_and(|n| n.parts[0].identity == "Schéma")
        })
        .unwrap();
    let span = relation.span.as_ref().unwrap();
    let slice = &source[span.start.offset..span.end.offset];
    assert!(slice.contains("Schéma"));
    assert!(q[0]
        .columns
        .iter()
        .filter(|c| c.name.sql.contains("\"B\""))
        .all(|c| c.span.is_some()));
}

mod edges;

mod exists_polarity;

mod modifying_ctes;
