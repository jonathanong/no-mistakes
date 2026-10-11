use super::{at, columns, stars, Locations, Relation};
use crate::codebase::postgres::idents::{ident_key, object_name_key};
use sqlparser::ast::{
    FromTable, Spanned, Statement, TableFactor, TableObject, TableWithJoins, UpdateTableFromKind,
};

pub(super) fn relation(factor: &TableFactor) -> Option<Relation> {
    if let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = factor
    {
        Some(Relation {
            table: object_name_key(name),
            alias: alias.as_ref().map(|alias| ident_key(&alias.name)),
            at: at(name.span()),
        })
    } else {
        None
    }
}
pub(super) fn relations(tables: &[TableWithJoins]) -> Vec<Relation> {
    let mut out = Vec::new();
    for table in tables {
        factor(&table.relation, &mut out);
        for join in &table.joins {
            factor(&join.relation, &mut out);
        }
    }
    out
}
fn factor(table: &TableFactor, out: &mut Vec<Relation>) {
    if let Some(relation) = relation(table) {
        out.push(relation);
    }
    if let TableFactor::NestedJoin {
        table_with_joins, ..
    } = table
    {
        out.extend(relations(std::slice::from_ref(table_with_joins)));
    }
}
pub(super) fn statement(index: &mut Locations, statement: &Statement) {
    index.writes.extend(super::writes::collect(statement));
    match statement {
        Statement::Insert(insert) => {
            if let TableObject::TableName(name) = &insert.table {
                let table = object_name_key(name);
                index.inserts.push((
                    crate::codebase::postgres::schema::relation_name(name),
                    at(statement.span()),
                ));
                returning(
                    index,
                    insert.returning.as_deref(),
                    vec![Relation {
                        table,
                        alias: insert
                            .table_alias
                            .as_ref()
                            .map(|alias| ident_key(&alias.alias)),
                        at: at(name.span()),
                    }],
                );
            }
        }
        Statement::Update(update) => {
            let mut tables = vec![update.table.clone()];
            if let Some(
                UpdateTableFromKind::BeforeSet(from) | UpdateTableFromKind::AfterSet(from),
            ) = &update.from
            {
                tables.extend_from_slice(from);
            }
            index
                .mutation_columns
                .extend(columns::mutation(&tables, update.selection.as_ref()));
            returning(index, update.returning.as_deref(), relations(&tables));
        }
        Statement::Delete(delete) => {
            let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) = &delete.from;
            let mut tables = from.clone();
            if let Some(using) = &delete.using {
                tables.extend_from_slice(using);
            }
            index
                .mutation_columns
                .extend(columns::mutation(&tables, delete.selection.as_ref()));
            returning(index, delete.returning.as_deref(), relations(&tables));
        }
        _ => {}
    }
}
fn returning(
    index: &mut Locations,
    items: Option<&[sqlparser::ast::SelectItem]>,
    relations: Vec<Relation>,
) {
    index.returning.extend(
        stars::items(items.unwrap_or_default())
            .into_iter()
            .map(|star| (star, relations.clone())),
    );
}
