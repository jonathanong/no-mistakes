use super::SqlSelectFact;
use sqlparser::ast::TableFactor;

pub(super) fn walk_factor(
    sql: &str,
    factor: &TableFactor,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    match factor {
        TableFactor::Derived { subquery, .. } => {
            super::super::select::collect_query(sql, subquery, ctes, false, false, selects);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => super::walk_side_queries(
            sql,
            std::slice::from_ref(table_with_joins),
            None,
            ctes,
            selects,
        ),
        _ => {}
    }
}
