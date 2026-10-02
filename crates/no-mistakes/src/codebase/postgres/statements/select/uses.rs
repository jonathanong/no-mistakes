use super::super::predicates::base_table;
use super::super::{SqlColumnClause, SqlColumnUseFact};
use sqlparser::ast::{BinaryOperator, Expr, OrderByExpr, Select, TableFactor, TableWithJoins};

struct BaseRel {
    table: String,
    alias: Option<String>,
}

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
        for join in &table.joins {
            if let Some(expr) = super::join_expr(&join.join_operator) {
                walk(expr, SqlColumnClause::Join, &rels, line, &mut out);
            }
        }
    }
    for item in order {
        record(&item.expr, SqlColumnClause::OrderBy, &rels, line, &mut out);
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
        clause,
        line,
    });
}

fn bare(expr: &Expr) -> Option<(Option<String>, String)> {
    match peel(expr) {
        Expr::Identifier(ident) => Some((None, ident.value.to_ascii_lowercase())),
        Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
            let column = parts.last()?.value.to_ascii_lowercase();
            let qualifier = parts[parts.len() - 2].value.to_ascii_lowercase();
            Some((Some(qualifier), column))
        }
        _ => None,
    }
}

fn resolve(rels: &[BaseRel], qualifier: Option<&str>) -> Option<String> {
    let Some(qualifier) = qualifier else {
        return if rels.len() == 1 {
            Some(rels[0].table.clone())
        } else {
            Some(String::new())
        };
    };
    rels.iter()
        .find(|rel| rel.alias.as_deref() == Some(qualifier))
        .or_else(|| rels.iter().find(|rel| rel.table == qualifier))
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
            | BinaryOperator::Lt
            | BinaryOperator::LtEq
            | BinaryOperator::Gt
            | BinaryOperator::GtEq
    )
}

fn base_relations(from: &[TableWithJoins], ctes: &[String]) -> Vec<BaseRel> {
    let mut rels = Vec::new();
    for table in from {
        push_factor(&table.relation, ctes, &mut rels);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut rels);
        }
    }
    rels
}

fn push_factor(factor: &TableFactor, ctes: &[String], rels: &mut Vec<BaseRel>) {
    match factor {
        TableFactor::Table { name, alias, .. } => {
            let Some(table) = base_table(name, ctes) else {
                return;
            };
            rels.push(BaseRel {
                table: table.to_ascii_lowercase(),
                alias: alias
                    .as_ref()
                    .map(|alias| alias.name.value.to_ascii_lowercase()),
            });
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => rels.extend(base_relations(std::slice::from_ref(table_with_joins), ctes)),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
