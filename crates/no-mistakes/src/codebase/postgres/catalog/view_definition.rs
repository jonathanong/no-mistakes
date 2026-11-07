use super::SchemaCatalog;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::parse_postgres_sql;
use dashmap::DashMap;
use sqlparser::ast::{Expr, Query, SelectItem, SetExpr, SetQuantifier, Statement};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) type ViewProjections = DashMap<String, Option<PreparedView>>;

#[derive(Debug)]
pub(super) struct PreparedView {
    query: Arc<Query>,
    columns: Option<Arc<BTreeSet<String>>>,
}

impl SchemaCatalog {
    pub(crate) fn view_query(&self, name: &str) -> Option<Arc<Query>> {
        self.project_view(name, |view| Some(view.query.clone()))
    }

    pub(crate) fn view_column_names(&self, name: &str) -> Option<Arc<BTreeSet<String>>> {
        self.project_view(name, |view| view.columns.clone())
    }

    fn project_view<T>(
        &self,
        name: &str,
        project: impl FnOnce(&PreparedView) -> Option<T>,
    ) -> Option<T> {
        let view = self.view_relation(name)?;
        let cache = self
            .projections()
            .view_projections
            .get_or_init(Default::default);
        let prepared = cache
            .entry(view.name.clone())
            .or_insert_with(|| prepare(&view.definition));
        project(prepared.as_ref()?)
    }
}

fn prepare(definition: &str) -> Option<PreparedView> {
    let mut statements = parse_postgres_sql(definition).ok()?;
    if statements.len() != 1 {
        return None;
    }
    let Statement::Query(query) = statements.pop()? else {
        return None;
    };
    Some(PreparedView {
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
