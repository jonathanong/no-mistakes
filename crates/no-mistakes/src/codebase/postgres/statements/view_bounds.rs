use super::{predicates::Projection, SqlSelectFact};
use crate::codebase::postgres::{CatalogView, PostgresParseError, SchemaCatalog};
use sqlparser::ast::{BinaryOperator, Expr};
use std::cell::Cell;

mod keys;
mod scalar;

pub(crate) const BOUNDED_VIEW_MARKER: &str = "__no_mistakes_bounded_view__";

pub(crate) fn bounded_view_selects(
    view: &CatalogView,
    functions: &[String],
    catalog: &SchemaCatalog,
) -> Result<Vec<SqlSelectFact>, PostgresParseError> {
    let query = catalog.view_query(&view.name)?;
    let definition = &view.definition;
    let projection = Bounds {
        functions,
        catalog,
        table_arm: Cell::new(false),
    };
    let mut selects = Vec::new();
    super::select::collect_query_projected(
        definition,
        &query,
        &[],
        false,
        false,
        super::select::policy(None, &projection),
        &mut selects,
    );
    if projection.table_arm.get() {
        return Err(unsupported());
    }
    Ok(selects)
}

fn unsupported() -> PostgresParseError {
    PostgresParseError {
        message:
            "expected a static SELECT definition without TABLE query arms or parenthesized joins"
                .into(),
    }
}

struct Bounds<'a> {
    functions: &'a [String],
    catalog: &'a SchemaCatalog,
    table_arm: Cell<bool>,
}

impl Projection for Bounds<'_> {
    fn relation_name(&self, name: &sqlparser::ast::ObjectName, _: String) -> String {
        crate::codebase::postgres::idents::object_name_identity(name)
    }

    fn qualifier_matches(&self, table: &str, qualifier: &str) -> bool {
        crate::codebase::postgres::idents::relation_suffix_name(table) == qualifier
    }

    fn nested_join(&self) {
        self.table_arm.set(true);
    }

    fn table_arm(&self) {
        self.table_arm.set(true);
    }

    fn map<'a>(
        &self,
        predicate: &Expr,
        other: Option<&Expr>,
        table: &str,
        column: String,
        proven: bool,
        peers: impl Iterator<Item = &'a str>,
    ) -> Option<(String, bool)> {
        let table = self.catalog.bounded_relation(table)?;
        if !proven
            && peers.into_iter().any(|peer| {
                self.catalog
                    .bounded_relation(peer)
                    .is_none_or(|peer| peer.columns.iter().any(|info| info.name == column))
            })
        {
            return None;
        }
        let accepted = match crate::codebase::postgres::idents::unwrap_expr(predicate) {
            Expr::BinaryOp {
                op: BinaryOperator::Eq,
                ..
            } => {
                other.is_some_and(|value| scalar::configured_function(value, self.functions))
                    && keys::key_column(table, &column)
            }
            Expr::BinaryOp {
                op:
                    BinaryOperator::Gt
                    | BinaryOperator::GtEq
                    | BinaryOperator::Lt
                    | BinaryOperator::LtEq,
                ..
            } => {
                other.is_some_and(|value| {
                    scalar::scalar(value, self.functions)
                        && (!scalar::uses_configured_function(value, self.functions)
                            || keys::key_column(table, &column))
                }) && keys::indexed_column(table, &column)
            }
            Expr::Between {
                low,
                high,
                negated: false,
                ..
            } => {
                scalar::scalar(low, self.functions)
                    && scalar::scalar(high, self.functions)
                    && (!(scalar::uses_configured_function(low, self.functions)
                        || scalar::uses_configured_function(high, self.functions))
                        || keys::key_column(table, &column))
                    && keys::indexed_column(table, &column)
            }
            _ => false,
        };
        accepted.then(|| (BOUNDED_VIEW_MARKER.to_string(), true))
    }
}
