use no_mistakes_postgres_sql::idents::{ident_key, object_name_key};
use sqlparser::ast::{
    FromTable, Query, Select, SetExpr, Statement, TableFactor, TableObject, TableWithJoins,
    UpdateTableFromKind,
};

mod exprs;
mod items;
use exprs::walk_subqueries;
use items::{collect_items, collect_returning, collect_table_expr, target_rel, visit_join_constraint};

pub(super) const SYNTHETIC_TARGET: &str = "__nm_frag_target";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FragmentStar {
    pub(super) returning: bool,
    pub(super) qualifier: Option<String>,
    pub(super) relation: Option<String>,
    pub(super) within_function: Option<String>,
    pub(super) line: usize,
}

#[derive(Clone)]
pub(super) struct Rel {
    pub(super) table: String,
    pub(super) alias: Option<String>,
    pub(super) is_local_derived: bool,
}

pub(super) fn collect_statements(statements: &[Statement], out: &mut Vec<FragmentStar>) {
    for statement in statements {
        collect_statement(statement, &[], out);
    }
}

fn collect_statement(statement: &Statement, ctes: &[String], out: &mut Vec<FragmentStar>) {
    match statement {
        Statement::Query(query) => collect_query(query, ctes, false, out),
        Statement::Insert(insert) => collect_insert(insert, ctes, out),
        Statement::Update(update) => collect_update(update, ctes, out),
        Statement::Delete(delete) => collect_delete(delete, ctes, out),
        Statement::Explain { statement, .. } => collect_statement(statement, ctes, out),
        _ => {}
    }
}

fn collect_insert(insert: &sqlparser::ast::Insert, ctes: &[String], out: &mut Vec<FragmentStar>) {
    if let TableObject::TableName(name) = &insert.table {
        let target = Rel {
            table: object_name_key(name),
            alias: insert.table_alias.as_ref().map(|a| ident_key(&a.alias)),
            is_local_derived: false,
        };
        collect_returning(&target, &[], ctes, insert.returning.as_deref(), out);
    }
    if let Some(source) = insert.source.as_deref() {
        collect_query(source, ctes, false, out);
    }
}

fn collect_update(update: &sqlparser::ast::Update, ctes: &[String], out: &mut Vec<FragmentStar>) {
    let Some(target) = target_rel(&update.table.relation) else {
        return;
    };
    let from = match &update.from {
        Some(UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables)) => {
            tables.as_slice()
        }
        None => &[],
    };
    let mut peers = collect_rels(from, ctes, out);
    for join in &update.table.joins {
        push_factor(&join.relation, ctes, &mut peers, out);
        visit_join_constraint(&join.join_operator, ctes, out);
    }
    collect_returning(&target, &peers, ctes, update.returning.as_deref(), out);
}

fn collect_delete(delete: &sqlparser::ast::Delete, ctes: &[String], out: &mut Vec<FragmentStar>) {
    let first = match &delete.from {
        FromTable::WithFromKeyword(t) | FromTable::WithoutKeyword(t) => t.first(),
    };
    if let Some(target) = first.and_then(|table| target_rel(&table.relation)) {
        let peers = collect_rels(delete.using.as_deref().unwrap_or_default(), ctes, out);
        collect_returning(&target, &peers, ctes, delete.returning.as_deref(), out);
    }
}

pub(super) fn collect_query(
    query: &Query,
    outer_ctes: &[String],
    in_exists: bool,
    out: &mut Vec<FragmentStar>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = ident_key(&cte.alias.name);
            if with.recursive {
                ctes.push(name.clone());
            }
            collect_query(&cte.query, &ctes, false, out);
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    collect_set(&query.body, &ctes, in_exists, out);
}

fn collect_set(expr: &SetExpr, ctes: &[String], in_exists: bool, out: &mut Vec<FragmentStar>) {
    match expr {
        SetExpr::Select(select) => collect_select(select, ctes, in_exists, out),
        SetExpr::Query(query) => collect_query(query, ctes, in_exists, out),
        SetExpr::Insert(stmt) | SetExpr::Update(stmt) | SetExpr::Delete(stmt) => {
            collect_statement(stmt, ctes, out);
        }
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(left, ctes, in_exists, out);
            collect_set(right, ctes, in_exists, out);
        }
        SetExpr::Table(table) if !in_exists => collect_table_expr(table, ctes, out),
        _ => {}
    }
}

fn collect_select(select: &Select, ctes: &[String], in_exists: bool, out: &mut Vec<FragmentStar>) {
    let rels = collect_rels(&select.from, ctes, out);
    if !in_exists {
        collect_items(&select.projection, &rels, ctes, false, out);
    }
    for expr in select
        .selection
        .iter()
        .chain(select.having.iter())
        .chain(select.qualify.iter())
        .chain(select.prewhere.iter())
    {
        walk_subqueries(expr, ctes, out);
    }
}

fn collect_rels(from: &[TableWithJoins], ctes: &[String], out: &mut Vec<FragmentStar>) -> Vec<Rel> {
    let mut rels = Vec::new();
    for table in from {
        push_factor(&table.relation, ctes, &mut rels, out);
        for join in &table.joins {
            push_factor(&join.relation, ctes, &mut rels, out);
            visit_join_constraint(&join.join_operator, ctes, out);
        }
    }
    rels
}

fn push_factor(factor: &TableFactor, ctes: &[String], rels: &mut Vec<Rel>, out: &mut Vec<FragmentStar>) {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => {
            let table = object_name_key(name);
            let is_cte = !table.contains('.')
                && ctes.iter().any(|cte| cte.eq_ignore_ascii_case(&table));
            rels.push(Rel {
                table,
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                is_local_derived: is_cte,
            });
        }
        TableFactor::Derived {
            subquery, alias, ..
        } => {
            collect_query(subquery, ctes, false, out);
            rels.push(Rel {
                table: String::new(),
                alias: alias.as_ref().map(|a| ident_key(&a.name)),
                is_local_derived: true,
            });
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => rels.extend(collect_rels(std::slice::from_ref(table_with_joins), ctes, out)),
        _ => {}
    }
}
