//! `ORDER BY` normalization: select aliases, positional references and constant keys.
use crate::codebase::postgres::{
    expression_is_constant, parse_postgres_expression, CanonicalOrderKey, SchemaCatalog,
    SqlConflictInsertFact, SqlInsertSourceShape,
};
use sqlparser::ast::Expr;

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

/// Whether the source reads one relation pinned to a single row by a catalog unique key.
pub(super) fn pins_one_row(insert: &SqlConflictInsertFact, catalog: &SchemaCatalog) -> bool {
    insert
        .source
        .pinned_relation
        .as_ref()
        .is_some_and(|pinned| catalog.columns_pin_one_row(&pinned.table, &pinned.columns))
}
