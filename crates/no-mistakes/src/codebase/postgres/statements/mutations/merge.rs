use super::SqlSelectFact;
use sqlparser::ast::Merge;

/// MERGE predicates and action subqueries share the existing shape/query walk.
pub(super) fn collect(
    sql: &str,
    statement: &Merge,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    super::walk_factor(sql, &statement.table, ctes, selects);
    super::walk_factor(sql, &statement.source, ctes, selects);
    super::walk_side_queries(sql, &[], Some(&statement.on), ctes, selects);
    for clause in &statement.clauses {
        super::walk_side_queries(sql, &[], clause.predicate.as_ref(), ctes, selects);
        super::super::select::walk_node(sql, &clause.action, ctes, false, selects);
    }
}
