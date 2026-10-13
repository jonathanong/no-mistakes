use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{
    EmbeddedSqlFileFacts, EmbeddedSqlOptions, PostgresSchemaOptions, SchemaCatalog,
    SqlStatementFileFacts,
};
use std::path::{Path, PathBuf};

pub(crate) struct PreparedSqlInputs<'a> {
    pub(crate) catalog: &'a SchemaCatalog,
    pub(crate) statements: Vec<&'a SqlStatementFileFacts>,
    pub(crate) embedded: Vec<&'a EmbeddedSqlFileFacts>,
}

pub(crate) fn collect_prepared_sql_inputs<'a>(
    root: &Path,
    files: &[PathBuf],
    schema: &PostgresSchemaOptions,
    catalog_path: &str,
    embedded_options: &EmbeddedSqlOptions,
    prepared: Option<&'a CheckFactMap>,
) -> anyhow::Result<PreparedSqlInputs<'a>> {
    let prepared =
        prepared.ok_or_else(|| anyhow::anyhow!("prepared PostgreSQL facts are required"))?;
    let catalog = prepared.postgres_schema_catalog(catalog_path)?;
    let paths = crate::codebase::postgres::postgres_sql_paths(root, files, schema)?;
    let mut statements = Vec::new();
    let mut embedded = Vec::new();
    for path in &paths {
        statements.extend(prepared.postgres_statements(path, None)?.iter());
    }
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        embedded.push(prepared.embedded_sql(path, embedded_options)?);
        statements.extend(
            prepared
                .postgres_statements(path, Some(embedded_options))?
                .iter(),
        );
    }
    Ok(PreparedSqlInputs {
        catalog,
        statements,
        embedded,
    })
}
