use super::super::helpers::{generated, generated_name};
use super::{and_keys, plain_assignments, plain_value, Key, PlaceholderPositions, RowAccess};
use crate::codebase::postgres::idents::{ident_key, object_name_identity};
use sqlparser::ast::{
    ConflictTarget, Insert, ObjectNamePart, OnConflict, OnConflictAction, OnInsert, Query, SetExpr,
    Spanned, TableObject,
};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) struct UpsertKeys {
    pub(super) keys: Vec<RowAccess>,
    pub(super) unconditional: bool,
}

pub(super) fn upsert_keys(
    insert: &Insert,
    positions: PlaceholderPositions<'_>,
    tokens: &[TokenWithSpan],
) -> Option<UpsertKeys> {
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
        || has_overriding_user_value(insert, source, tokens)
        || has_conflict_target_predicate(source, tokens)
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
    let unconditional = update.selection.is_none();
    if let Some(predicate) = &update.selection {
        and_keys(predicate, &qualifier, positions, &mut Vec::new())?;
    }
    let target_index = insert.columns.iter().position(|name| {
        matches!(name.0.as_slice(), [ObjectNamePart::Identifier(id)] if ident_key(id) == column)
    })?;
    let keys = values
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
        .collect::<Option<Vec<_>>>()?;
    Some(UpsertKeys {
        keys,
        unconditional,
    })
}

fn has_overriding_user_value(insert: &Insert, source: &Query, tokens: &[TokenWithSpan]) -> bool {
    let start = insert.insert_token.0.span.start;
    let end = source.span().start;
    let first = tokens.partition_point(|token| token.span.start < start);
    let last = tokens.partition_point(|token| token.span.start < end);
    let (mut prev2, mut prev1) = (None::<&str>, None::<&str>);
    for token in &tokens[first..last] {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        if let Token::Word(word) = &token.token {
            if word.quote_style.is_none() {
                if prev2.is_some_and(|w| w.eq_ignore_ascii_case("overriding"))
                    && prev1.is_some_and(|w| w.eq_ignore_ascii_case("user"))
                    && word.value.eq_ignore_ascii_case("value")
                {
                    return true;
                }
                (prev2, prev1) = (prev1, Some(word.value.as_str()));
                continue;
            }
        }
        (prev2, prev1) = (None, None);
    }
    false
}

fn has_conflict_target_predicate(source: &Query, tokens: &[TokenWithSpan]) -> bool {
    let start = source.span().end;
    let first = tokens.partition_point(|token| token.span.start < start);
    let mut depth = 0usize;
    for token in &tokens[first..] {
        match &token.token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            Token::Word(word) if depth == 0 && word.quote_style.is_none() => {
                if word.value.eq_ignore_ascii_case("where") {
                    return true;
                }
                if word.value.eq_ignore_ascii_case("do") {
                    return false;
                }
            }
            Token::SemiColon if depth == 0 => return false,
            _ => {}
        }
    }
    false
}
