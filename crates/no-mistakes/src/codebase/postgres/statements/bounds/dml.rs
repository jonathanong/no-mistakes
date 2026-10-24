use super::items::Builder;
use super::{Scope, SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use crate::codebase::postgres::statements::SqlPinSource;
use sqlparser::ast::{Delete, FromTable, Statement, Update, UpdateTableFromKind};

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
        statement_start: None,
        query: SqlBoundQuery {
            input_mode: Default::default(),
            capped,
            items,
            outputs: Vec::new(),
        },
        target: Some(0),
    });
}

/// RETURNING emits one row per changed target row. Keep only pins independent of
/// auxiliary FROM/USING items when projecting that cardinality into a CTE.
pub(super) fn returning_bound(
    statement: &Statement,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> Option<SqlBoundQuery> {
    let mut facts = Vec::new();
    // The statement's own fact is collected later. This projection must not consume its
    // shared source-token cursor while inspecting auxiliary subqueries.
    let mut projection_scope = scope.clone();
    projection_scope.table_tokens = None;
    match statement {
        Statement::Update(update)
            if update
                .returning
                .as_ref()
                .is_some_and(|items| !items.is_empty()) =>
        {
            self::update(update, &projection_scope, positions, &mut facts);
        }
        Statement::Delete(delete)
            if delete
                .returning
                .as_ref()
                .is_some_and(|items| !items.is_empty()) =>
        {
            self::delete(delete, &projection_scope, positions, &mut facts);
        }
        _ => return None,
    }
    let query = facts.pop()?.query;
    let mut target = query.items.into_iter().next()?;
    target
        .pins
        .retain(|pin| matches!(&pin.source, SqlPinSource::Value | SqlPinSource::Query(_)));
    Some(SqlBoundQuery {
        input_mode: query.input_mode,
        capped: query.capped,
        items: vec![target],
        outputs: Vec::new(),
    })
}
