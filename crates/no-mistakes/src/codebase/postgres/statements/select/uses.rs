use super::super::{SqlColumnClause, SqlColumnUseFact};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{
    BinaryOperator, Expr, OrderByExpr, Select, SelectItem, Spanned, TableWithJoins,
};

mod scope;
use scope::{base_relations, push_factor, BaseRel};

pub(super) fn collect(
    select: &Select,
    order: &[OrderByExpr],
    ctes: &[String],
    line: usize,
) -> Vec<SqlColumnUseFact> {
    let rels = base_relations(&select.from, ctes);
    let mut out = Vec::new();
    if let Some(selection) = &select.selection {
        walk(selection, SqlColumnClause::Where, &rels, line, &mut out);
    }
    for table in &select.from {
        let mut visible = Vec::new();
        push_factor(&table.relation, ctes, &mut visible);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut visible);
            if let Some(expr) = super::join_expr(&join.join_operator) {
                walk(expr, SqlColumnClause::Join, &visible, line, &mut out);
            }
        }
    }
    for item in order {
        let expr = match peel(&item.expr) {
            Expr::Identifier(alias) => select
                .projection
                .iter()
                .find_map(|projection| {
                    if let SelectItem::ExprWithAlias { expr, alias: name } = projection {
                        (ident_key(name) == ident_key(alias)).then_some(expr)
                    } else {
                        None
                    }
                })
                .unwrap_or(&item.expr),
            _ => &item.expr,
        };
        let start = out.len();
        record(expr, SqlColumnClause::OrderBy, &rels, line, &mut out);
        for use_ in &mut out[start..] {
            use_.line = expression_line(&item.expr, line);
        }
    }
    out
}

pub(super) fn collect_mutation(
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
) -> Vec<SqlColumnUseFact> {
    let rels = base_relations(tables, ctes);
    let mut out = Vec::new();
    if let Some(expr) = selection {
        walk(expr, SqlColumnClause::Where, &rels, 1, &mut out);
    }
    for table in tables {
        let mut visible = Vec::new();
        push_factor(&table.relation, ctes, &mut visible);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut visible);
            if let Some(expr) = super::join_expr(&join.join_operator) {
                walk(expr, SqlColumnClause::Join, &visible, 1, &mut out);
            }
        }
    }
    out
}

fn walk(
    expr: &Expr,
    clause: SqlColumnClause,
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlColumnUseFact>,
) {
    let expr = peel(expr);
    match expr {
        Expr::BinaryOp { left, op, right } if is_logic(op) => {
            walk(left, clause, rels, line, out);
            walk(right, clause, rels, line, out);
        }
        Expr::BinaryOp { left, op, right } if is_comparison(op) => {
            record(left, clause, rels, line, out);
            record(right, clause, rels, line, out);
        }
        Expr::Between { expr, .. } => record(expr, clause, rels, line, out),
        Expr::InList { expr, .. } | Expr::InSubquery { expr, .. } => {
            record(expr, clause, rels, line, out);
        }
        Expr::UnaryOp {
            op: sqlparser::ast::UnaryOperator::Not,
            expr,
        } => walk(expr, clause, rels, line, out),
        _ => {}
    }
}

fn record(
    expr: &Expr,
    clause: SqlColumnClause,
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlColumnUseFact>,
) {
    let Some((qualifier, column)) = bare(expr) else {
        return;
    };
    let Some(table) = resolve(rels, qualifier.as_deref()) else {
        return;
    };
    out.push(SqlColumnUseFact {
        table,
        column,
        candidate_tables: (!rels.iter().any(|rel| rel.unknown))
            .then(|| rels.iter().map(|rel| rel.table.clone()).collect()),
        clause,
        line: expression_line(expr, line),
    });
}

fn bare(expr: &Expr) -> Option<(Option<String>, String)> {
    match peel(expr) {
        Expr::Identifier(ident) => Some((None, ident_key(ident))),
        Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
            let column = ident_key(parts.last()?);
            let qualifier = parts[..parts.len() - 1]
                .iter()
                .map(ident_key)
                .collect::<Vec<_>>()
                .join(".");
            Some((Some(qualifier), column))
        }
        _ => None,
    }
}

fn resolve(rels: &[BaseRel], qualifier: Option<&str>) -> Option<String> {
    let Some(qualifier) = qualifier else {
        return if rels.len() == 1 && !rels[0].unknown {
            Some(rels[0].table.clone())
        } else {
            Some(String::new())
        };
    };
    rels.iter()
        .find(|rel| rel.alias.as_deref() == Some(qualifier))
        .or_else(|| {
            rels.iter().find(|rel| {
                rel.table == qualifier || rel.table.rsplit('.').next() == Some(qualifier)
            })
        })
        .map(|rel| rel.table.clone())
}

fn peel(expr: &Expr) -> &Expr {
    let mut expr = expr;
    while let Expr::Nested(inner) = expr {
        expr = inner;
    }
    expr
}

fn is_logic(op: &BinaryOperator) -> bool {
    matches!(op, BinaryOperator::And | BinaryOperator::Or)
}

fn is_comparison(op: &BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Eq
            | BinaryOperator::NotEq
            | BinaryOperator::Lt
            | BinaryOperator::LtEq
            | BinaryOperator::Gt
            | BinaryOperator::GtEq
    )
}

fn expression_line(expr: &Expr, fallback: usize) -> usize {
    let line = expr.span().start.line as usize;
    if line == 0 {
        fallback
    } else {
        line
    }
}

#[cfg(test)]
mod tests;
