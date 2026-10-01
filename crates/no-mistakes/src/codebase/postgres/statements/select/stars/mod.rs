mod dml;

use super::super::predicates::base_table;
use super::super::SqlStarProjectionFact;
use crate::codebase::postgres::idents::visit_child_exprs;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectName, Select,
    SelectItem, SelectItemQualifiedWildcardKind, TableFactor, TableWithJoins,
};

pub(super) fn returning(
    sql: &str,
    statement: &sqlparser::ast::Statement,
) -> Vec<SqlStarProjectionFact> {
    dml::returning(sql, statement)
}

struct BaseRel {
    table: String,
    alias: Option<String>,
}

pub(super) fn collect(select: &Select, ctes: &[String], line: usize) -> Vec<SqlStarProjectionFact> {
    let rels = base_relations(&select.from, ctes);
    let mut out = Vec::new();
    collect_items(&select.projection, &rels, line, &mut out);
    out
}

pub(super) fn collect_one(
    table: &str,
    alias: Option<&str>,
    items: &[SelectItem],
    line: usize,
) -> Vec<SqlStarProjectionFact> {
    let rels = vec![BaseRel {
        table: table.to_ascii_lowercase(),
        alias: alias.map(str::to_ascii_lowercase),
    }];
    let mut out = Vec::new();
    collect_items(items, &rels, line, &mut out);
    out
}

fn collect_items(
    items: &[SelectItem],
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    for item in items {
        match item {
            SelectItem::Wildcard(_) => {
                for rel in rels {
                    push(out, &rel.table, false, None, line);
                }
            }
            SelectItem::QualifiedWildcard(kind, _) => {
                if let SelectItemQualifiedWildcardKind::ObjectName(name) = kind {
                    record_qualified(rels, name, None, line, out);
                }
            }
            SelectItem::UnnamedExpr(expr)
            | SelectItem::ExprWithAlias { expr, .. }
            | SelectItem::ExprWithAliases { expr, .. } => walk_expr(expr, rels, line, out),
        }
    }
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
        } => {
            rels.extend(base_relations(std::slice::from_ref(table_with_joins), ctes));
        }
        _ => {}
    }
}

fn walk_expr(expr: &Expr, rels: &[BaseRel], line: usize, out: &mut Vec<SqlStarProjectionFact>) {
    if let Expr::Function(function) = expr {
        walk_function(function, rels, line, out);
        return;
    }
    visit_child_exprs(expr, &mut |child| walk_expr(child, rels, line, out));
}

fn walk_function(
    function: &Function,
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    let FunctionArguments::List(list) = &function.args else {
        return;
    };
    let name = relation_name(&function.name).to_ascii_lowercase();
    for arg in &list.args {
        let Some(expr) = arg_expr(arg) else {
            continue;
        };
        match expr {
            FunctionArgExpr::QualifiedWildcard(object) => {
                record_qualified(rels, object, Some(name.clone()), line, out);
            }
            FunctionArgExpr::Expr(expr) => walk_expr(expr, rels, line, out),
            FunctionArgExpr::Wildcard | FunctionArgExpr::WildcardWithOptions(_) => {}
        }
    }
}

fn arg_expr(arg: &FunctionArg) -> Option<&FunctionArgExpr> {
    match arg {
        FunctionArg::Unnamed(expr)
        | FunctionArg::Named { arg: expr, .. }
        | FunctionArg::ExprNamed { arg: expr, .. } => Some(expr),
    }
}

fn record_qualified(
    rels: &[BaseRel],
    name: &ObjectName,
    within_function: Option<String>,
    line: usize,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    let Some(rel) = resolve(rels, name) else {
        return;
    };
    push(out, &rel.table, true, within_function, line);
}

fn resolve<'a>(rels: &'a [BaseRel], name: &ObjectName) -> Option<&'a BaseRel> {
    let ident = relation_name(name).to_ascii_lowercase();
    if ident.is_empty() {
        return None;
    }
    rels.iter()
        .find(|rel| rel.alias.as_deref() == Some(ident.as_str()))
        .or_else(|| rels.iter().find(|rel| rel.table == ident))
}

fn push(
    out: &mut Vec<SqlStarProjectionFact>,
    relation: &str,
    qualified: bool,
    within_function: Option<String>,
    line: usize,
) {
    out.push(SqlStarProjectionFact {
        relation: relation.to_string(),
        qualified,
        within_function,
        line,
    });
}

#[cfg(test)]
mod tests;
