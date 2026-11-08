use super::SchemaCatalog;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::{parse_postgres_sql, PostgresParseError};
use dashmap::DashMap;
use sqlparser::ast::{Expr, Query, SelectItem, SetExpr, SetQuantifier, Statement};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) type ViewProjections = DashMap<String, Result<PreparedView, PostgresParseError>>;

#[derive(Debug)]
pub(super) struct PreparedView {
    query: Arc<Query>,
    columns: Option<Arc<BTreeSet<String>>>,
}

impl SchemaCatalog {
    pub(crate) fn view_query(&self, name: &str) -> Option<Arc<Query>> {
        self.project_view(name, |view| {
            view.as_ref().ok().map(|view| Arc::clone(&view.query))
        })
        .flatten()
    }

    pub(crate) fn try_view_query(&self, name: &str) -> Result<Arc<Query>, PostgresParseError> {
        self.project_view(name, |view| {
            view.as_ref()
                .map(|view| Arc::clone(&view.query))
                .map_err(Clone::clone)
        })
        .unwrap_or_else(|| {
            Err(PostgresParseError {
                message: format!("unknown catalog view {name}"),
            })
        })
    }

    pub(crate) fn view_column_names(&self, name: &str) -> Option<Arc<BTreeSet<String>>> {
        self.project_view(name, |view| {
            view.as_ref().ok().and_then(|view| view.columns.clone())
        })
        .flatten()
    }

    fn project_view<T>(
        &self,
        name: &str,
        project: impl FnOnce(&Result<PreparedView, PostgresParseError>) -> T,
    ) -> Option<T> {
        let view = self.view_relation(name)?;
        let cache = self
            .projections()
            .view_projections
            .get_or_init(Default::default);
        let prepared = cache
            .entry(view.name.clone())
            .or_insert_with(|| prepare(&view.definition));
        Some(project(&prepared))
    }
}

fn prepare(definition: &str) -> Result<PreparedView, PostgresParseError> {
    let mut statements = parse_postgres_sql(definition)?.into_iter();
    let query = match (statements.next(), statements.next()) {
        (Some(Statement::Query(query)), None) => query,
        _ => {
            return Err(PostgresParseError {
                message: "expected one static SELECT view definition".into(),
            })
        }
    };
    Ok(PreparedView {
        columns: query_columns(&query).map(Arc::new),
        query: Arc::from(query),
    })
}

fn query_columns(query: &Query) -> Option<BTreeSet<String>> {
    query
        .pipe_operators
        .is_empty()
        .then(|| columns(&query.body))
        .flatten()
}

fn columns(body: &SetExpr) -> Option<BTreeSet<String>> {
    match body {
        SetExpr::Select(select) => select.projection.iter().map(column_name).collect(),
        SetExpr::Query(query) => query_columns(query),
        SetExpr::SetOperation {
            left,
            set_quantifier,
            ..
        } if !matches!(
            set_quantifier,
            SetQuantifier::ByName | SetQuantifier::AllByName | SetQuantifier::DistinctByName
        ) =>
        {
            columns(left)
        }
        _ => None,
    }
}

fn column_name(item: &SelectItem) -> Option<String> {
    match item {
        SelectItem::ExprWithAlias { alias, .. } => Some(ident_key(alias)),
        SelectItem::UnnamedExpr(expr) => match unwrap_expr(expr) {
            Expr::Identifier(ident) => Some(ident_key(ident)),
            Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
            _ => None,
        },
        _ => None,
    }
}
