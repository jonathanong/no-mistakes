//! `ORDER BY` normalization: select aliases, positional references and constant keys.
use crate::codebase::postgres::{
    expression_is_constant, normalize_catalog_identifier, parse_postgres_expression,
    CanonicalOrderKey, SchemaCatalog, SqlConflictInsertFact, SqlInsertSourceShape,
};
use sqlparser::ast::Expr;
use std::collections::BTreeSet;

/// Replace a select alias or an integer position with the select-list expression it names.
pub(super) fn resolve_references(
    order: &[CanonicalOrderKey],
    source: &SqlInsertSourceShape,
) -> Vec<CanonicalOrderKey> {
    order
        .iter()
        .map(|key| CanonicalOrderKey {
            expression: source
                .order_aliases
                .get(&key.expression.to_ascii_lowercase())
                .cloned()
                .or_else(|| positional(&key.expression, source))
                .or_else(|| qualified(&key.expression, source))
                .unwrap_or_else(|| key.expression.clone()),
            ascending: key.ascending,
            nulls_first: key.nulls_first,
        })
        .collect()
}

fn positional(expression: &str, source: &SqlInsertSourceShape) -> Option<String> {
    let position: usize = expression.parse().ok()?;
    source
        .select_list
        .as_ref()?
        .get(position.checked_sub(1)?)
        .cloned()
}

/// A bare column name resolves to `relation.column` only when exactly one relation in scope
/// can supply it: the sole relation, or the only one whose declared columns include the name.
/// A relation with unknown columns (a plain table) makes a multi-relation scope ambiguous.
fn qualified(expression: &str, source: &SqlInsertSourceShape) -> Option<String> {
    let name = match parse_postgres_expression(expression)? {
        Expr::Identifier(ident) => ident.value.to_ascii_lowercase(),
        _ => return None,
    };
    let relations = source.relations.as_ref()?;
    let owner = match relations.as_slice() {
        [only] => only,
        _ => {
            let mut owners = relations.iter().filter(|relation| {
                relation
                    .columns
                    .as_ref()
                    .is_none_or(|columns| columns.contains(&name))
            });
            let owner = owners.next()?;
            if owners.next().is_some() || owner.columns.is_none() {
                return None;
            }
            owner
        }
    };
    let qualified = format!("{}.{name}", owner.qualifier.as_ref()?);
    // Only a select-list expression of that exact shape can be the one the key names.
    source
        .select_list
        .as_ref()?
        .iter()
        .find(|item| item.eq_ignore_ascii_case(&qualified))
        .cloned()
}

/// Drop keys that are literals, bound parameters, or select-list expressions the source
/// proved to be recovered template placeholders.
pub(super) fn without_constants(
    keys: &[CanonicalOrderKey],
    source: &SqlInsertSourceShape,
) -> Vec<CanonicalOrderKey> {
    keys.iter()
        .filter(|key| {
            !expression_is_constant(&key.expression)
                && !source.constant_projections.contains(&key.expression)
        })
        .cloned()
        .collect()
}

/// Whether the source's inner-joined relations are each pinned to at most one row by a catalog
/// unique key via constant equalities and join equalities.
pub(super) fn pins_one_row(insert: &SqlConflictInsertFact, catalog: &SchemaCatalog) -> bool {
    let Some(source) = insert.source.pinned_source.as_ref() else {
        return false;
    };
    let mut proven = vec![false; source.relations.len()];
    let mut pinned: Vec<BTreeSet<String>> = source
        .relations
        .iter()
        .map(|relation| {
            relation
                .columns
                .iter()
                .map(|column| normalize_catalog_identifier(column))
                .collect()
        })
        .collect();
    let unique_keys: Vec<Vec<Vec<String>>> = source
        .relations
        .iter()
        .map(|relation| catalog.unique_keys(&relation.table))
        .collect();
    let equalities: Vec<(usize, String, usize, String)> = source
        .join_equalities
        .iter()
        .map(|eq| {
            (
                eq.left_relation,
                normalize_catalog_identifier(&eq.left_column),
                eq.right_relation,
                normalize_catalog_identifier(&eq.right_column),
            )
        })
        .collect();
    loop {
        let mut changed = false;
        for (index, keys) in unique_keys.iter().enumerate() {
            if !proven[index]
                && keys
                    .iter()
                    .any(|key| !key.is_empty() && key.iter().all(|col| pinned[index].contains(col)))
            {
                proven[index] = true;
                changed = true;
            }
        }
        for (left, left_col, right, right_col) in &equalities {
            if (proven[*left] || pinned[*left].contains(left_col))
                && pinned[*right].insert(right_col.clone())
            {
                changed = true;
            }
            if (proven[*right] || pinned[*right].contains(right_col))
                && pinned[*left].insert(left_col.clone())
            {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    !proven.is_empty() && proven.into_iter().all(|is_proven| is_proven)
}

pub(super) fn display_keys(keys: &[CanonicalOrderKey]) -> String {
    keys.iter()
        .map(|key| {
            format!(
                "{} {} NULLS {}",
                key.expression,
                if key.ascending { "ASC" } else { "DESC" },
                if key.nulls_first { "FIRST" } else { "LAST" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}
