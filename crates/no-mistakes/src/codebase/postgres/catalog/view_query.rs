use super::{SchemaCatalog, ViewQueryResult};
use crate::codebase::postgres::{parse_postgres_sql, PostgresParseError};
use sqlparser::ast::Statement;
use std::sync::Arc;

impl SchemaCatalog {
    pub(crate) fn view_query(&self, name: &str) -> ViewQueryResult {
        let view = self.view_relation(name).ok_or_else(|| PostgresParseError {
            message: format!("unknown catalog view {name}"),
        })?;
        self.view_queries
            .get_or_init(dashmap::DashMap::new)
            .entry(view.name.clone())
            .or_insert_with(|| {
                let statements = parse_postgres_sql(&view.definition)?;
                let mut statements = statements.into_iter();
                match (statements.next(), statements.next()) {
                    (Some(Statement::Query(query)), None) => Ok(Arc::from(query)),
                    _ => Err(PostgresParseError {
                        message: "expected one static SELECT view definition".into(),
                    }),
                }
            })
            .clone()
    }
}
