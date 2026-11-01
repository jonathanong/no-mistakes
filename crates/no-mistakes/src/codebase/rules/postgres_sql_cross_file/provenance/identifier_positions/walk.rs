use super::Indexes;
use sqlparser::ast::{
    AssignmentTarget, Expr, FromTable, GroupByExpr, Ident, ObjectName, ObjectNamePart, OrderByExpr,
    OrderByKind, SelectItem, SetExpr, Statement, TableFactor, TableObject, TableWithJoins,
    UpdateTableFromKind,
};
use std::collections::BTreeSet;

pub(super) fn walk_statement(statement: &Statement, found: &mut Indexes) {
    match statement {
        Statement::Query(query) => walk_query(query, found),
        Statement::Insert(insert) => {
            note_names(&insert.columns, &mut found.columns);
            note_table_object(&insert.table, found);
            if let Some(source) = insert.source.as_deref() {
                walk_query(source, found);
            }
        }
        Statement::Update(update) => {
            note_assignments(&update.assignments, found);
            walk_joined(&update.table, found);
            if let Some(from) = &update.from {
                let tables = match from {
                    UpdateTableFromKind::BeforeSet(tables)
                    | UpdateTableFromKind::AfterSet(tables) => tables,
                };
                walk_tables(tables, found);
            }
            note_order(&update.order_by, found);
        }
        Statement::Delete(delete) => {
            let tables = match &delete.from {
                FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables,
            };
            walk_tables(tables, found);
            if let Some(using) = &delete.using {
                walk_tables(using, found);
            }
            note_order(&delete.order_by, found);
        }
        _ => {}
    }
}

fn walk_query(query: &sqlparser::ast::Query, found: &mut Indexes) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            walk_query(&cte.query, found);
        }
    }
    walk_set(&query.body, found);
    if let Some(OrderByKind::Expressions(exprs)) = query.order_by.as_ref().map(|order| &order.kind)
    {
        note_order(exprs, found);
    }
}

fn walk_set(expr: &SetExpr, found: &mut Indexes) {
    match expr {
        SetExpr::Select(select) => {
            for item in &select.projection {
                if let SelectItem::UnnamedExpr(expr)
                | SelectItem::ExprWithAlias { expr, .. }
                | SelectItem::ExprWithAliases { expr, .. } = item
                {
                    note_bare(expr, &mut found.columns);
                }
            }
            if let GroupByExpr::Expressions(exprs, _) = &select.group_by {
                for expr in exprs {
                    note_bare(expr, &mut found.columns);
                }
            }
            walk_tables(&select.from, found);
        }
        SetExpr::Query(query) => walk_query(query, found),
        SetExpr::SetOperation { left, right, .. } => {
            walk_set(left, found);
            walk_set(right, found);
        }
        SetExpr::Insert(statement) | SetExpr::Update(statement) | SetExpr::Delete(statement) => {
            walk_statement(statement, found);
        }
        _ => {}
    }
}

fn walk_tables(tables: &[TableWithJoins], found: &mut Indexes) {
    for table in tables {
        walk_joined(table, found);
    }
}

fn walk_joined(table: &TableWithJoins, found: &mut Indexes) {
    walk_factor(&table.relation, found);
    for join in &table.joins {
        walk_factor(&join.relation, found);
    }
}

fn walk_factor(factor: &TableFactor, found: &mut Indexes) {
    if let TableFactor::Table { name, .. } = factor {
        note_name(name, &mut found.relations);
    }
    match factor {
        TableFactor::Derived { subquery, .. } => walk_query(subquery, found),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => walk_joined(table_with_joins, found),
        _ => {}
    }
}

fn note_table_object(table: &TableObject, found: &mut Indexes) {
    match table {
        TableObject::TableName(name) => note_name(name, &mut found.relations),
        TableObject::TableQuery(query) => walk_query(query, found),
        _ => {}
    }
}

fn note_assignments(assignments: &[sqlparser::ast::Assignment], found: &mut Indexes) {
    for assignment in assignments {
        match &assignment.target {
            AssignmentTarget::ColumnName(name) => note_name(name, &mut found.columns),
            AssignmentTarget::Tuple(names) => note_names(names, &mut found.columns),
        }
    }
}

fn note_order(exprs: &[OrderByExpr], found: &mut Indexes) {
    for ordering in exprs {
        note_bare(&ordering.expr, &mut found.columns);
    }
}

fn note_names(names: &[ObjectName], out: &mut BTreeSet<u32>) {
    for name in names {
        note_name(name, out);
    }
}

pub(super) fn note_name(name: &ObjectName, out: &mut BTreeSet<u32>) {
    for part in &name.0 {
        if let ObjectNamePart::Identifier(ident) = part {
            note_ident(ident, out);
        }
    }
}

fn note_bare(expr: &Expr, out: &mut BTreeSet<u32>) {
    match expr {
        Expr::Identifier(ident) => note_ident(ident, out),
        Expr::CompoundIdentifier(parts) => {
            for ident in parts {
                note_ident(ident, out);
            }
        }
        Expr::Nested(inner) => note_bare(inner, out),
        _ => {}
    }
}

/// Every placeholder in the identifier, so `sql_placeholder_1__votes` and
/// `audit_sql_placeholder_1` still mark hole 1 as occupying the name.
fn note_ident(ident: &Ident, out: &mut BTreeSet<u32>) {
    let mut rest = ident.value.as_str();
    while let Some(at) = rest.find(MARKER) {
        let after = &rest[at + MARKER.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if let Ok(index) = after[..digits].parse::<u32>() {
            if index > 0 {
                out.insert(index);
            }
        }
        rest = &after[digits..];
    }
}

const MARKER: &str = "sql_placeholder_";
