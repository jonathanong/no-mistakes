use super::CheckFactMap;
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlSchemaFileFacts, SqlStatementFileFacts};
use std::path::Path;

impl CheckFactMap {
    // A recorded read/parse failure is tolerated; a missing demand remains an error.
    pub(crate) fn postgres_readable_schema(
        &self,
        path: &Path,
    ) -> anyhow::Result<Option<&SqlSchemaFileFacts>> {
        self.postgres.readable_schema(path)
    }
    pub(crate) fn postgres_readable_statements(
        &self,
        path: &Path,
        profile: Option<&EmbeddedSqlOptions>,
    ) -> anyhow::Result<Option<&[SqlStatementFileFacts]>> {
        self.postgres.readable_statements(path, profile)
    }
}
