use super::{predicates::Projection, SqlSelectFact};
use crate::codebase::postgres::{CatalogView, PostgresParseError, SchemaCatalog};
use sqlparser::ast::{BinaryOperator, Expr, Query, TableFactor};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

mod constants;
mod joins;
mod keys;
mod queries;
mod scalar;

type LookupProof = fn(&mut [super::SqlBoundItem], &mut [bool], &SchemaCatalog);

pub(crate) const BOUNDED_VIEW_MARKER: &str = "__no_mistakes_bounded_view__";

pub(crate) fn bounded_view_selects(
    view: &CatalogView,
    functions: &[String],
    catalog: &SchemaCatalog,
    lookup: LookupProof,
) -> Result<Vec<SqlSelectFact>, PostgresParseError> {
    let query = catalog.try_view_query(&view.name)?;
    let query = constants::inline(&query, functions);
    let definition = &view.definition;
    let projection = Bounds {
        functions,
        catalog,
        unsupported: Cell::new(false),
        queries: RefCell::new(HashSet::new()),
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
    if projection.unsupported.get() || !queries::complete(&query, &projection.queries.borrow()) {
        return Err(unsupported());
    }
    joins::project(
        &super::bounds::lookup_query(&query),
        &mut selects,
        catalog,
        lookup,
    );
    Ok(selects)
}

fn unsupported() -> PostgresParseError {
    PostgresParseError {
        message:
            "expected a static SELECT definition without TABLE query arms, parenthesized joins, unsupported FROM factors (functions, sampling, renamed table columns), or unsupported subquery clauses"
                .into(),
    }
}

struct Bounds<'a> {
    functions: &'a [String],
    catalog: &'a SchemaCatalog,
    unsupported: Cell<bool>,
    queries: RefCell<HashSet<*const Query>>,
}

impl Projection for Bounds<'_> {
    fn relation_name(&self, name: &sqlparser::ast::ObjectName, _: String) -> String {
        crate::codebase::postgres::idents::object_name_identity(name)
    }

    fn qualifier_matches(&self, table: &str, qualifier: &str) -> bool {
        crate::codebase::postgres::idents::relation_suffix_name(table) == qualifier
    }

    fn table_arm(&self) {
        self.unsupported.set(true);
    }

    fn inspect_query(&self, query: &Query) {
        self.queries.borrow_mut().insert(std::ptr::from_ref(query));
    }

    fn inspect_factor(&self, factor: &TableFactor) {
        let supported = match factor {
            TableFactor::Table {
                name: _,
                args: None,
                sample: None,
                version: None,
                json_path: None,
                with_ordinality: false,
                with_hints,
                partitions,
                index_hints,
                alias,
            } => {
                with_hints.is_empty()
                    && partitions.is_empty()
                    && index_hints.is_empty()
                    && alias.as_ref().is_none_or(|alias| alias.columns.is_empty())
            }
            TableFactor::Derived {
                lateral: _,
                subquery: _,
                alias: _,
                sample: None,
            } => true,
            _ => false,
        };
        if !supported {
            self.unsupported.set(true);
        }
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
