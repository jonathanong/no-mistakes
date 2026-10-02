use super::{base_relations, collect_items, BaseRel};
use crate::codebase::postgres::idents::{ident_key, object_name_key};
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use sqlparser::ast::{
    FromTable, Query, SelectItem, SetExpr, Statement, TableFactor, TableObject, TableWithJoins,
    UpdateTableFromKind,
};

pub(super) fn returning(sql: &str, statement: &Statement) -> Vec<SqlStarProjectionFact> {
    let mut out = Vec::new();
    collect(sql, statement, &[], &mut out);
    out
}

fn collect(
    sql: &str,
    statement: &Statement,
    ctes: &[String],
    out: &mut Vec<SqlStarProjectionFact>,
) {
    match statement {
        Statement::Query(query) => collect_query(sql, query, ctes, out),
        Statement::Insert(insert) => {
            if let TableObject::TableName(name) = &insert.table {
                let target = BaseRel {
                    table: object_name_key(name),
                    alias: insert
                        .table_alias
                        .as_ref()
                        .map(|alias| ident_key(&alias.alias)),
                };
                emit(&target, &[], insert.returning.as_deref(), out);
            }
            if let Some(source) = insert.source.as_deref() {
                collect_query(sql, source, ctes, out);
            }
        }
        Statement::Update(update) => {
            if let Some(target) = target_factor(&update.table.relation) {
                let from = match &update.from {
                    Some(
                        UpdateTableFromKind::BeforeSet(tables)
                        | UpdateTableFromKind::AfterSet(tables),
                    ) => tables.as_slice(),
                    None => &[],
                };
                let mut peers = base_relations(from, ctes);
                for join in &update.table.joins {
                    super::push_factor(&join.relation, ctes, &mut peers);
                }
                emit(&target, &peers, update.returning.as_deref(), out);
            }
        }
        Statement::Delete(delete) => {
            if let Some(table) = delete_target(&delete.from) {
                if let Some(target) = target_factor(&table.relation) {
                    let peers = base_relations(delete.using.as_deref().unwrap_or_default(), ctes);
                    emit(&target, &peers, delete.returning.as_deref(), out);
                }
            }
        }
        Statement::Explain { statement, .. } => collect(sql, statement, ctes, out),
        _ => {}
    }
}

fn collect_query(
    sql: &str,
    query: &Query,
    outer_ctes: &[String],
    out: &mut Vec<SqlStarProjectionFact>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = ident_key(&cte.alias.name);
            if with.recursive {
                ctes.push(name.clone());
            }
            collect_query(sql, &cte.query, &ctes, out);
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    collect_set(sql, &query.body, &ctes, out);
}

fn collect_set(sql: &str, expr: &SetExpr, ctes: &[String], out: &mut Vec<SqlStarProjectionFact>) {
    match expr {
        SetExpr::Insert(statement) | SetExpr::Update(statement) | SetExpr::Delete(statement) => {
            collect(sql, statement, ctes, out)
        }
        SetExpr::Query(query) => collect_query(sql, query, ctes, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(sql, left, ctes, out);
            collect_set(sql, right, ctes, out);
        }
        _ => {}
    }
}

fn emit(
    target: &BaseRel,
    peers: &[BaseRel],
    items: Option<&[SelectItem]>,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    let Some(items) = items else {
        return;
    };
    let rels: Vec<_> = std::iter::once(target)
        .chain(peers.iter())
        .cloned()
        .collect();
    for item in items {
        // Bare RETURNING * expands only the mutation target; qualified stars see FROM/USING peers.
        let visible = if matches!(item, SelectItem::Wildcard(_)) {
            std::slice::from_ref(target)
        } else {
            &rels
        };
        collect_items(std::slice::from_ref(item), visible, 1, out);
    }
}

fn delete_target(from: &FromTable) -> Option<&TableWithJoins> {
    match from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables.first(),
    }
}

fn target_factor(factor: &TableFactor) -> Option<BaseRel> {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => Some(BaseRel {
            table: object_name_key(name),
            alias: alias.as_ref().map(|alias| ident_key(&alias.name)),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
