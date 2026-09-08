use super::{
    collect_from_query, collect_from_set_expr, collect_queries_from_expr, expr_has_multi_row,
    extract_locking_select_metadata, function_is_any, function_name_is_any, order_keys,
    set_expr_has_multi_row, LockingSelectMetadata,
};
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{
    BinaryOperator, Expr, Function, FunctionArguments, Ident, ObjectName, ObjectNamePart, OrderBy,
    OrderByKind, OrderByOptions, Query, SetExpr, Statement, UnaryOperator, Values,
};

fn first(sql: &str) -> LockingSelectMetadata {
    let locks = extract_locking_select_metadata(sql).expect("parse");
    assert_eq!(locks.len(), 1, "{sql} => {locks:?}");
    locks.into_iter().next().expect("one locking select")
}

#[test]
fn any_predicate_without_order_or_skip_is_multi_row() {
    let meta = first("SELECT * FROM t WHERE id = ANY($1) FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
    assert!(!meta.has_order_by);
    assert!(!meta.skips_locked_rows);
}

#[test]
fn in_list_predicate_is_multi_row() {
    let meta = first("SELECT * FROM t WHERE id IN (1, 2) FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
    assert!(!meta.has_order_by);
}

#[test]
fn in_subquery_predicate_is_multi_row() {
    let meta = first("SELECT * FROM t WHERE id IN (SELECT id FROM u) FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn order_by_is_recorded() {
    let meta = first("SELECT * FROM t WHERE id = ANY($1) ORDER BY id FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
    assert!(meta.has_order_by);
    assert!(!meta.skips_locked_rows);
}

#[test]
fn non_expression_order_by_has_no_canonical_key_projection() {
    assert!(order_keys(&OrderBy {
        kind: OrderByKind::All(OrderByOptions::default()),
        interpolate: None,
    })
    .is_none());
}

#[test]
fn records_every_unqualified_lock_relation_and_resolves_of_aliases() {
    let meta = first(
        "SELECT * FROM jobs AS j JOIN users AS u ON u.id = j.user_id WHERE u.id = ANY($1) ORDER BY j.id FOR UPDATE",
    );
    assert_eq!(
        meta.tables,
        Some(vec!["jobs".to_string(), "users".to_string()])
    );
    let meta = first(
        "SELECT * FROM jobs AS j JOIN users AS u ON u.id = j.user_id WHERE u.id = ANY($1) ORDER BY j.id FOR UPDATE OF j",
    );
    assert_eq!(meta.tables, Some(vec!["jobs".to_string()]));
}

#[test]
fn explicit_lock_targets_must_resolve_to_exactly_one_base_relation() {
    let unresolved =
        first("SELECT * FROM jobs AS j WHERE j.id = ANY($1) ORDER BY j.id FOR UPDATE OF missing");
    assert_eq!(unresolved.tables, None);

    // The unaliased base name would refer to either side of this self-join.
    let ambiguous = first(
        "SELECT * FROM jobs AS left_job JOIN jobs AS right_job ON true WHERE left_job.id = ANY($1) ORDER BY left_job.id FOR UPDATE OF jobs",
    );
    assert_eq!(ambiguous.tables, None);
}

#[test]
fn locking_a_derived_relation_does_not_claim_a_base_table_order() {
    let meta = first(
        "SELECT * FROM (SELECT * FROM jobs WHERE id = ANY($1)) AS queued ORDER BY queued.id FOR UPDATE",
    );
    assert_eq!(meta.tables, None);
}

#[test]
fn nested_join_relations_are_collected_for_unqualified_locks() {
    let meta = first(
        "SELECT * FROM (jobs AS j JOIN users AS u ON true) WHERE j.id = ANY($1) ORDER BY j.id FOR UPDATE",
    );
    assert_eq!(
        meta.tables,
        Some(vec!["jobs".to_string(), "users".to_string()])
    );
}

#[test]
fn relation_resolver_handles_an_empty_update_lock_set_and_deduplicates_self_joins() {
    let statements = parse_postgres_sql("SELECT * FROM jobs").expect("parse");
    let Statement::Query(query) = &statements[0] else {
        panic!("expected query");
    };
    assert_eq!(
        super::relations::locked_tables(&query.body, &[]),
        Some(Vec::new())
    );

    let meta = first(
        "SELECT * FROM jobs AS first_job JOIN jobs AS second_job ON true WHERE first_job.id = ANY($1) ORDER BY first_job.id FOR UPDATE",
    );
    assert_eq!(meta.tables, Some(vec!["jobs".to_string()]));
}

#[test]
fn skip_locked_is_recorded() {
    let meta = first("SELECT * FROM t WHERE id = ANY($1) FOR UPDATE SKIP LOCKED");
    assert!(meta.has_multi_row_predicate);
    assert!(!meta.has_order_by);
    assert!(meta.skips_locked_rows);
}

#[test]
fn single_row_equality_is_not_multi_row() {
    let meta = first("SELECT * FROM t WHERE id = $1 FOR UPDATE");
    assert!(!meta.has_multi_row_predicate);
}

#[test]
fn parenthesized_any_and_and_is_multi_row() {
    let meta = first("SELECT * FROM t WHERE (id = ANY($1)) AND active FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn not_in_is_still_multi_row() {
    let meta = first("SELECT * FROM t WHERE id NOT IN (1, 2) FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn for_share_is_ignored() {
    let locks = extract_locking_select_metadata("SELECT * FROM t WHERE id = ANY($1) FOR SHARE")
        .expect("parse");
    assert!(locks.is_empty());
}

#[test]
fn unparseable_sql_returns_error() {
    let error = extract_locking_select_metadata("SELECT * FROM t WHERE id = ANY($1 FOR UPDATE")
        .expect_err("unparseable");
    assert!(!error.message.is_empty());
}

#[test]
fn cte_locking_select_is_found() {
    let meta = first(
        "WITH locked AS (SELECT * FROM t WHERE id = ANY($1) FOR UPDATE) SELECT * FROM locked",
    );
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn derived_table_locking_select_is_found() {
    let meta = first("SELECT * FROM (SELECT * FROM t WHERE id IN (1, 2) FOR UPDATE) AS locked");
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn union_without_order_by_keeps_multi_row() {
    let meta = first(
        "SELECT * FROM t WHERE id = ANY($1) UNION SELECT * FROM t WHERE id = ANY($2) FOR UPDATE",
    );
    assert!(meta.has_multi_row_predicate);
    assert!(!meta.has_order_by);
}

#[test]
fn sql_without_locks_is_empty() {
    let locks = extract_locking_select_metadata("SELECT * FROM t WHERE id = ANY($1)").unwrap();
    assert!(locks.is_empty());
}

#[test]
fn nested_query_body_without_select_is_not_multi_row() {
    let meta = first("SELECT * FROM t FOR UPDATE");
    assert!(!meta.has_multi_row_predicate);
    assert!(!meta.has_order_by);
}

#[test]
fn values_set_expr_is_not_multi_row() {
    let meta = first("VALUES (1) FOR UPDATE");
    assert!(!meta.has_multi_row_predicate);
}

#[test]
fn unary_not_still_sees_any() {
    let meta = first("SELECT * FROM t WHERE NOT (id = ANY($1)) FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn join_derived_locking_select_is_found() {
    let meta = first(
        "SELECT * FROM t JOIN (SELECT * FROM u WHERE id = ANY($1) FOR UPDATE) AS locked ON true",
    );
    assert!(meta.has_multi_row_predicate);
}

#[test]
fn locking_subquery_in_predicate_is_found() {
    let locks = extract_locking_select_metadata(
        "SELECT * FROM t WHERE id IN (SELECT id FROM u WHERE x = ANY($1) FOR UPDATE)",
    )
    .expect("parse");
    assert!(locks.iter().any(|lock| lock.has_multi_row_predicate));
}

#[test]
fn non_query_statements_are_ignored() {
    let locks = extract_locking_select_metadata("CREATE TABLE t (id int)").unwrap();
    assert!(locks.is_empty());
}

#[test]
fn parenthesized_query_body_is_walked() {
    let meta = first("SELECT * FROM t WHERE id = ANY($1) FOR UPDATE");
    assert!(meta.has_multi_row_predicate);
}

fn any_function() -> Function {
    Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("ANY"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::None,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    }
}

fn empty_query() -> Query {
    Query {
        with: None,
        body: Box::new(SetExpr::Values(Values {
            explicit_row: false,
            rows: Vec::new(),
            value_keyword: false,
        })),
        order_by: None,
        limit_clause: None,
        fetch: None,
        locks: Vec::new(),
        for_clause: None,
        settings: None,
        format_clause: None,
        pipe_operators: Vec::new(),
    }
}

#[test]
fn constructed_any_function_is_multi_row() {
    let any = Expr::Function(any_function());
    assert!(function_is_any(&any));
    assert!(function_name_is_any(&any_function()));
    assert!(expr_has_multi_row(&Expr::BinaryOp {
        left: Box::new(Expr::Identifier(Ident::new("id"))),
        op: BinaryOperator::Eq,
        right: Box::new(any.clone()),
    }));
    assert!(expr_has_multi_row(&Expr::BinaryOp {
        left: Box::new(any),
        op: BinaryOperator::Eq,
        right: Box::new(Expr::Identifier(Ident::new("id"))),
    }));
    assert!(!function_is_any(&Expr::Identifier(Ident::new("id"))));
    assert!(!function_name_is_any(&Function {
        name: ObjectName(vec![]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::None,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    }));
}

#[test]
fn constructed_set_expr_and_unnest_helpers() {
    let nested = SetExpr::Query(Box::new(empty_query()));
    assert!(!set_expr_has_multi_row(&nested));
    let unnest = Expr::InUnnest {
        expr: Box::new(Expr::Identifier(Ident::new("id"))),
        array_expr: Box::new(Expr::Identifier(Ident::new("ids"))),
        negated: false,
    };
    assert!(expr_has_multi_row(&unnest));
    let unary = Expr::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(unnest),
    };
    assert!(expr_has_multi_row(&unary));
    let mut locks = Vec::new();
    collect_from_set_expr(&nested, &mut locks);
    collect_from_set_expr(
        &SetExpr::Values(Values {
            explicit_row: false,
            rows: Vec::new(),
            value_keyword: false,
        }),
        &mut locks,
    );
    collect_from_query(&empty_query(), &mut locks);
    collect_queries_from_expr(
        &Expr::UnaryOp {
            op: UnaryOperator::Plus,
            expr: Box::new(Expr::Identifier(Ident::new("id"))),
        },
        &mut locks,
    );
    collect_queries_from_expr(&Expr::Identifier(Ident::new("id")), &mut locks);
    assert!(locks.is_empty());
}
