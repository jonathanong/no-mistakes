use super::SqlSelectFact;
use sqlparser::ast::{Merge, MergeAction, MergeInsertKind, MergeUpdateKind};

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
        collect_action_shapes(sql, &clause.action, selects);
        super::super::select::walk_node(sql, &clause.action, ctes, false, selects);
    }
}

fn collect_action_shapes(sql: &str, action: &MergeAction, selects: &mut Vec<SqlSelectFact>) {
    match action {
        MergeAction::Update(update) => {
            if let MergeUpdateKind::Set(assignments) = &update.kind {
                for assignment in assignments {
                    action_expr(sql, &assignment.value, selects);
                }
            }
        }
        MergeAction::Insert(insert) => {
            if let MergeInsertKind::Values(values) = &insert.kind {
                for row in &values.rows {
                    for expr in &row.content {
                        action_expr(sql, expr, selects);
                    }
                }
            }
        }
        MergeAction::Delete { .. } | MergeAction::DoNothing { .. } => {}
    }
}

fn action_expr(sql: &str, expr: &sqlparser::ast::Expr, selects: &mut Vec<SqlSelectFact>) {
    super::super::select::collect_predicate_shapes(expr, selects);
    let mut exists_set_operations = Vec::new();
    super::super::exists::collect_exists(sql, Some(expr), &mut exists_set_operations);
    if exists_set_operations.is_empty() {
        return;
    }
    selects.push(SqlSelectFact {
        line: 1,
        tables: Vec::new(),
        predicate_sql: String::new(),
        exists_set_operations,
        relations: Vec::new(),
        in_insert_select: false,
        not_in_subqueries: Vec::new(),
        not_in_columns: Vec::new(),
        count_existence_checks: Vec::new(),
        star_projections: Vec::new(),
        column_uses: Vec::new(),
    });
}
