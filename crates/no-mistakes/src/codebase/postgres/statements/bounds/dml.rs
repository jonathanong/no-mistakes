use super::items::Builder;
use super::{Scope, SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use sqlparser::ast::{Delete, FromTable, Update, UpdateTableFromKind};

/// An `UPDATE` is bounded when the relation it changes is: other FROM items only feed it values.
pub(super) fn update(update: &Update, scope: &Scope, out: &mut Vec<SqlBoundFact>) {
    let mut builder = Builder::new(scope);
    builder.tables(std::slice::from_ref(&update.table));
    if let Some(UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables)) =
        &update.from
    {
        builder.tables(tables);
    }
    push(
        SqlBoundKind::Update,
        builder.finish(update.selection.as_ref()),
        update.limit.is_some(),
        out,
    );
}

pub(super) fn delete(delete: &Delete, scope: &Scope, out: &mut Vec<SqlBoundFact>) {
    let mut builder = Builder::new(scope);
    let (FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables)) = &delete.from;
    builder.tables(tables);
    if let Some(using) = &delete.using {
        builder.tables(using);
    }
    push(
        SqlBoundKind::Delete,
        builder.finish(delete.selection.as_ref()),
        delete.limit.is_some(),
        out,
    );
}

fn push(
    kind: SqlBoundKind,
    items: Vec<super::super::SqlBoundItem>,
    capped: bool,
    out: &mut Vec<SqlBoundFact>,
) {
    let line = items.first().map_or(1, |item| item.line);
    out.push(SqlBoundFact {
        kind,
        line,
        query: SqlBoundQuery { capped, items },
        target: Some(0),
    });
}
