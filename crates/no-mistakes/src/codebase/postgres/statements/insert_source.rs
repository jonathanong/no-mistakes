use super::value::{form_is_stable, from_expr};
use super::{SqlAssignmentFact, SqlValueForm};
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{Insert, Query, Select, SelectItem, SetExpr, Values};

pub(super) fn from_insert(insert: &Insert) -> Vec<SqlAssignmentFact> {
    let columns: Vec<String> = insert.columns.iter().map(relation_name).collect();
    if columns.is_empty() {
        return Vec::new();
    }
    insert
        .source
        .as_deref()
        .map(|query| from_query(&columns, query))
        .unwrap_or_default()
}

fn from_query(columns: &[String], query: &Query) -> Vec<SqlAssignmentFact> {
    from_set_expr(columns, &query.body)
}

fn from_set_expr(columns: &[String], expr: &SetExpr) -> Vec<SqlAssignmentFact> {
    match expr {
        SetExpr::Values(values) => from_values(columns, values),
        SetExpr::Select(select) => from_select(columns, select),
        SetExpr::Query(query) => from_query(columns, query),
        _ => Vec::new(),
    }
}

fn from_values(columns: &[String], values: &Values) -> Vec<SqlAssignmentFact> {
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let forms: Vec<SqlValueForm> = values
                .rows
                .iter()
                .map(|row| row.get(index).map(from_expr).unwrap_or(SqlValueForm::Other))
                .collect();
            SqlAssignmentFact {
                column: column.clone(),
                form: merge_forms(&forms),
            }
        })
        .collect()
}

fn from_select(columns: &[String], select: &Select) -> Vec<SqlAssignmentFact> {
    let Some(exprs) = select_exprs(select) else {
        return Vec::new();
    };
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| SqlAssignmentFact {
            column: column.clone(),
            form: exprs
                .get(index)
                .map(|expr| from_expr(expr))
                .unwrap_or(SqlValueForm::Other),
        })
        .collect()
}

fn select_exprs(select: &Select) -> Option<Vec<&sqlparser::ast::Expr>> {
    select
        .projection
        .iter()
        .map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .collect()
}

fn merge_forms(forms: &[SqlValueForm]) -> SqlValueForm {
    if let Some(form) = forms.iter().find(|form| !form_is_stable(form)) {
        return form.clone();
    }
    forms.first().cloned().unwrap_or(SqlValueForm::Other)
}
