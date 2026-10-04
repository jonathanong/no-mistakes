use super::items::Builder;
use super::{Scope, SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use sqlparser::ast::{Delete, FromTable, Update, UpdateTableFromKind};

/// An `UPDATE` is bounded when the relation it changes is: other FROM items only feed it values.
pub(super) fn update(
    update: &Update,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    let mut builder = Builder::new(scope, positions);
    builder.target_tables(std::slice::from_ref(&update.table));
    if let Some(UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables)) =
        &update.from
    {
        builder.tables(tables);
    }
    push(
        SqlBoundKind::Update,
        super::start(update.update_token.0.span),
        builder.finish(update.selection.as_ref()),
        update.limit.is_some() || super::predicate::rejects_all(update.selection.as_ref()),
        out,
    );
}

pub(super) fn delete(
    delete: &Delete,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    let mut builder = Builder::new(scope, positions);
    let (FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables)) = &delete.from;
    builder.target_tables(tables);
    if let Some(using) = &delete.using {
        builder.tables(using);
    }
    push(
        SqlBoundKind::Delete,
        super::start(delete.delete_token.0.span),
        builder.finish(delete.selection.as_ref()),
        delete.limit.is_some() || super::predicate::rejects_all(delete.selection.as_ref()),
        out,
    );
}

fn push(
    kind: SqlBoundKind,
    (line, column): (usize, usize),
    items: Vec<super::super::SqlBoundItem>,
    capped: bool,
    out: &mut Vec<SqlBoundFact>,
) {
    out.push(SqlBoundFact {
        kind,
        line,
        column,
        query: SqlBoundQuery { capped, items },
        target: Some(0),
    });
}
