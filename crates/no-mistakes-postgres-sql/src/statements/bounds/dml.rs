use super::items::Builder;
use super::{Scope, SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use crate::statements::SqlBoundInputMode;
use sqlparser::ast::{Delete, FromTable, Query, Statement, Update, UpdateTableFromKind};

/// An `UPDATE` is bounded when the relation it changes is: other FROM items only feed it values.
pub(super) fn update(
    update: &Update,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    out.push(update_fact(update, scope, positions));
}

fn update_fact(
    update: &Update,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundFact {
    let mut builder = Builder::new(scope, positions);
    builder.target_tables(std::slice::from_ref(&update.table));
    if let Some(UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables)) =
        &update.from
    {
        builder.tables(tables);
    }
    let items = builder.finish(update.selection.as_ref());
    scope.expressions_at(
        update,
        &items,
        || update.update_token.0.span.start,
        positions,
    );
    build_fact(
        SqlBoundKind::Update,
        super::start(update.update_token.0.span),
        items,
        update.limit.is_some() || super::predicate::rejects_all(update.selection.as_ref()),
    )
}

pub(super) fn delete(
    delete: &Delete,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    out.push(delete_fact(delete, scope, positions));
}

fn delete_fact(
    delete: &Delete,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundFact {
    let mut builder = Builder::new(scope, positions);
    let (FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables)) = &delete.from;
    builder.target_tables(tables);
    if let Some(using) = &delete.using {
        builder.tables(using);
    }
    let items = builder.finish(delete.selection.as_ref());
    scope.expressions_at(
        delete,
        &items,
        || delete.delete_token.0.span.start,
        positions,
    );
    build_fact(
        SqlBoundKind::Delete,
        super::start(delete.delete_token.0.span),
        items,
        delete.limit.is_some() || super::predicate::rejects_all(delete.selection.as_ref()),
    )
}

fn build_fact(
    kind: SqlBoundKind,
    (line, column): (usize, usize),
    items: Vec<super::super::SqlBoundItem>,
    capped: bool,
) -> SqlBoundFact {
    SqlBoundFact {
        body: None,
        kind,
        line,
        column,
        statement_start: None,
        query: SqlBoundQuery {
            input_mode: Default::default(),
            capped,
            items,
            outputs: Vec::new(),
            group_constant: false,
            group_keys: Vec::new(),
            catalog_limit: None,
        },
        target: Some(0),
        scalar_reads: None,
        lateral_qualified_reads: Vec::new(),
    }
}

/// RETURNING keeps target and driver evidence for catalog-backed target-row evaluation.
pub(super) fn returning_bound(
    query: &Query,
    statement: &Statement,
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> Option<SqlBoundQuery> {
    // The statement's own fact is collected later. This projection must not consume its
    // shared source-token cursor while inspecting auxiliary subqueries.
    let mut projection_scope = scope.clone();
    projection_scope.table_tokens = None;
    let projection_scope = super::query::with_scope(query, &projection_scope, positions);
    // RETURNING yields one row per inserted source row. The source query, not the target
    // table, is what a later statement can pin.
    if let Statement::Insert(insert) = statement {
        let returning = insert
            .returning
            .as_deref()
            .filter(|items| !items.is_empty())?;
        let source = insert.source.as_deref()?;
        let mut bound = super::query::bound_query(source, &projection_scope, positions);
        super::inserted::name(&mut bound, source, returning, positions);
        // Skipped and Blocking already decide the source. Streaming is the insert's row source.
        if bound.input_mode == SqlBoundInputMode::Streaming {
            bound.input_mode = SqlBoundInputMode::ReturningSource;
        }
        return Some(bound);
    }
    let (fact, returning) = match statement {
        Statement::Update(update) => {
            let returning = update
                .returning
                .as_deref()
                .filter(|items| !items.is_empty())?;
            (update_fact(update, &projection_scope, positions), returning)
        }
        Statement::Delete(delete) => {
            let returning = delete
                .returning
                .as_deref()
                .filter(|items| !items.is_empty())?;
            (delete_fact(delete, &projection_scope, positions), returning)
        }
        _ => return None,
    };
    super::returning::project(fact.query, returning)
}
