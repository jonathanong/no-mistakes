pub use no_mistakes_postgres_sql::types::*;

use super::embedded::EmbeddedSqlFileFacts;

/// Combined schema and embedded-SQL facts for one request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostgresFacts {
    pub schema: Vec<SqlSchemaFileFacts>,
    pub embedded: Vec<EmbeddedSqlFileFacts>,
    pub statements: Vec<crate::codebase::postgres::statement_facts::SqlStatementFileFacts>,
}
