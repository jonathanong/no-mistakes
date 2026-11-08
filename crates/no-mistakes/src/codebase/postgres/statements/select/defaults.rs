use super::super::{predicates::Identity, value::PlaceholderPositions, SqlSelectFact};
use super::projection::policy;
use sqlparser::ast::{Expr, Query, Visit};

pub(in crate::codebase::postgres::statements) fn collect_query_at(
    sql: &str,
    query: &Query,
    ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    super::collector::collect_query_projected(
        sql,
        query,
        ctes,
        in_insert_select,
        in_exists,
        policy(positions, &Identity),
        out,
    );
}

pub(in crate::codebase::postgres::statements) fn walk_expr_at(
    sql: &str,
    expr: &Expr,
    ctes: &[String],
    in_insert_select: bool,
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    super::nested::walk_expr_projected(
        sql,
        expr,
        ctes,
        in_insert_select,
        policy(positions, &Identity),
        out,
    );
}

pub(in crate::codebase::postgres::statements) fn walk_node_at<T: Visit>(
    sql: &str,
    node: &T,
    ctes: &[String],
    in_insert_select: bool,
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    super::nested::walk_node_projected(
        sql,
        node,
        ctes,
        in_insert_select,
        policy(positions, &Identity),
        out,
    );
}
