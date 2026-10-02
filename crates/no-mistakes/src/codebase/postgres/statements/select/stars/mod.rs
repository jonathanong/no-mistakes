mod dml;

use super::super::predicates::base_table;
use super::super::SqlStarProjectionFact;
use crate::codebase::postgres::idents::{ident_key, object_name_key};
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectName, Select,
    SelectItem, SelectItemQualifiedWildcardKind, Spanned, TableFactor, TableWithJoins,
};

pub(super) fn returning(
    sql: &str,
    statement: &sqlparser::ast::Statement,
) -> Vec<SqlStarProjectionFact> {
    dml::returning(sql, statement)
}

#[derive(Clone)]
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

fn collect_items(
    items: &[SelectItem],
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    for item in items {
        let line = span_line(item.span(), line);
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
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => {
            let Some(table) = base_table(name, ctes) else {
                return;
            };
            rels.push(BaseRel {
                table,
                alias: alias.as_ref().map(|alias| ident_key(&alias.name)),
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
    use sqlparser::ast::{Visit, Visitor};
    struct Functions<'a> {
        rels: &'a [BaseRel],
        line: usize,
        out: &'a mut Vec<SqlStarProjectionFact>,
        query_depth: usize,
    }
    impl Visitor for Functions<'_> {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
            if self.query_depth == 0 {
                if let Expr::Function(function) = expr {
                    walk_function(
                        function,
                        self.rels,
                        span_line(expr.span(), self.line),
                        self.out,
                    );
                }
            }
            std::ops::ControlFlow::Continue(())
        }
        fn pre_visit_query(&mut self, _: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
            self.query_depth += 1;
            std::ops::ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
            self.query_depth -= 1;
            std::ops::ControlFlow::Continue(())
        }
    }
    let _ = expr.visit(&mut Functions {
        rels,
        line,
        out,
        query_depth: 0,
    });
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
        let expr = arg_expr(arg);
        match expr {
            FunctionArgExpr::QualifiedWildcard(object) => {
                record_qualified(
                    rels,
                    object,
                    Some(name.clone()),
                    span_line(object.span(), line),
                    out,
                );
            }
            FunctionArgExpr::Expr(_) => {}
            FunctionArgExpr::Wildcard | FunctionArgExpr::WildcardWithOptions(_) => {}
        }
    }
}

fn span_line(span: sqlparser::tokenizer::Span, fallback: usize) -> usize {
    if span.start == span.end || span.start.line == 0 {
        fallback.max(1)
    } else {
        span.start.line as usize
    }
}

fn arg_expr(arg: &FunctionArg) -> &FunctionArgExpr {
    match arg {
        FunctionArg::Unnamed(expr)
        | FunctionArg::Named { arg: expr, .. }
        | FunctionArg::ExprNamed { arg: expr, .. } => expr,
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
    let ident = object_name_key(name);
    if ident.is_empty() {
        return None;
    }
    rels.iter()
        .find(|rel| rel.alias.as_deref() == Some(ident.as_str()))
        .or_else(|| {
            rels.iter().find(|rel| {
                rel.alias.is_none()
                    && (rel.table == ident || rel.table.rsplit('.').next() == Some(ident.as_str()))
            })
        })
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
