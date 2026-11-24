use super::super::helpers::{generated, generated_name};
use super::{and_keys, plain_assignments, plain_value, Key, PlaceholderPositions, RowAccess};
use crate::codebase::postgres::idents::{ident_key, object_name_identity};
use sqlparser::ast::{
    ConflictTarget, Insert, ObjectNamePart, OnConflict, OnConflictAction, OnInsert, SetExpr,
    TableObject,
};

pub(super) fn upsert_keys(
    insert: &Insert,
    positions: PlaceholderPositions<'_>,
) -> Option<Vec<RowAccess>> {
    let TableObject::TableName(relation) = &insert.table else {
        return None;
    };
    let Some(OnInsert::OnConflict(OnConflict {
        conflict_target:
            Some(ConflictTarget::Columns {
                columns,
                predicate: None,
            }),
        action: OnConflictAction::DoUpdate(update),
    })) = &insert.on
    else {
        return None;
    };
    let [key] = columns.as_slice() else {
        return None;
    };
    let source = insert.source.as_ref()?;
    let SetExpr::Values(values) = source.body.as_ref() else {
        return None;
    };
    if source.with.is_some()
        || source.limit_clause.is_some()
        || source.fetch.is_some()
        || generated_name(relation, positions)
        || generated(key, positions)
        || !plain_assignments(&update.assignments)
    {
        return None;
    }
    let relation = object_name_identity(relation);
    let column = ident_key(key);
    let qualifier = insert
        .table_alias
        .as_ref()
        .map_or_else(|| relation.clone(), |alias| ident_key(&alias.alias));
    if let Some(predicate) = &update.selection {
        and_keys(predicate, &qualifier, positions, &mut Vec::new())?;
    }
    let target_index = insert.columns.iter().position(|name| {
        matches!(name.0.as_slice(), [ObjectNamePart::Identifier(id)] if ident_key(id) == column)
    })?;
    values
        .rows
        .iter()
        .map(|row| {
            if row.content.len() != insert.columns.len()
                || !row.content.iter().all(|value| plain_value(value).is_some())
            {
                return None;
            }
            let value = plain_value(&row.content[target_index])?.to_string();
            Some(
                Key {
                    column: column.clone(),
                    value,
                }
                .row(relation.clone()),
            )
        })
        .collect()
}
