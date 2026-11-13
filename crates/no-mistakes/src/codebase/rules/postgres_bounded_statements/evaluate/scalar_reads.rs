use super::reads_outer;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::{
    SchemaCatalog, SqlBoundItemKind, SqlBoundPin, SqlBoundQuery, SqlPinSource, SqlScalarReads,
};
use sqlparser::ast::{BinaryOperator, Expr};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

type OuterColumns<'a> = BTreeMap<&'a str, BTreeSet<&'a str>>;

pub(super) fn project<'a>(
    query: &'a SqlBoundQuery,
    scalar: Option<&SqlScalarReads>,
    catalog: &SchemaCatalog,
) -> Cow<'a, SqlBoundQuery> {
    let Some(scalar) = scalar else {
        return Cow::Borrowed(query);
    };
    let mut columns = BTreeMap::new();
    for read in &scalar.reads {
        let outer = scalar.outer_columns.contains(&read.column)
            || scalar.outer_tables.iter().any(|(name, aliases)| {
                catalog.bounded_relation(name).is_some_and(|table| {
                    super::super::arrays::column_type(table, aliases, &read.column).is_some()
                })
            });
        if outer && reads_outer(std::slice::from_ref(read), catalog) {
            for table in &read.tables {
                columns
                    .entry(table.as_str())
                    .or_insert_with(BTreeSet::new)
                    .insert(read.column.as_str());
            }
        }
    }
    if columns.is_empty() {
        return Cow::Borrowed(query);
    }
    project_columns(query, &columns, catalog)
}

fn project_columns<'a>(
    query: &'a SqlBoundQuery,
    columns: &OuterColumns<'_>,
    catalog: &SchemaCatalog,
) -> Cow<'a, SqlBoundQuery> {
    let mut projected = Cow::Borrowed(query);
    for (index, item) in query.items.iter().enumerate() {
        match &item.kind {
            SqlBoundItemKind::Query(inner) => {
                if let Cow::Owned(inner) = project_columns(inner, columns, catalog) {
                    projected.to_mut().items[index].kind = SqlBoundItemKind::Query(inner);
                }
            }
            SqlBoundItemKind::Table(name) => {
                let (Some(outer), Some(table)) =
                    (columns.get(name.as_str()), catalog.bounded_relation(name))
                else {
                    continue;
                };
                for predicate in &item.predicate_conjuncts {
                    let Expr::BinaryOp {
                        left,
                        op: BinaryOperator::Eq,
                        right,
                    } = unwrap_expr(predicate)
                    else {
                        continue;
                    };
                    for (local, value) in [
                        (left.as_ref(), right.as_ref()),
                        (right.as_ref(), left.as_ref()),
                    ] {
                        let (Expr::Identifier(local), Expr::Identifier(value)) =
                            (unwrap_expr(local), unwrap_expr(value))
                        else {
                            continue;
                        };
                        let local = ident_key(local);
                        let value = ident_key(value);
                        if item.column_aliases.contains(&value)
                            || item.alias.as_ref() == Some(&value)
                            || !outer.contains(value.as_str())
                            || super::super::arrays::column_type(
                                table,
                                &item.column_aliases,
                                &local,
                            )
                            .is_none()
                        {
                            continue;
                        }
                        let pin = SqlBoundPin {
                            alternative: None,
                            column: local,
                            unqualified_candidate: false,
                            expression: None,
                            source: SqlPinSource::Value,
                            null_safe: false,
                            reads: Vec::new(),
                            qualified_reads: Vec::new(),
                            peer_column: None,
                        };
                        if !projected.items[index].pins.contains(&pin) {
                            projected.to_mut().items[index].pins.push(pin);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    projected
}

#[cfg(test)]
mod tests;
