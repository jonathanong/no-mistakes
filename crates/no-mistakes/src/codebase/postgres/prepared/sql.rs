//! Native SQL projections from one prepared parse per selected source.
use super::*;
use rayon::prelude::*;

pub(super) fn collect(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    plan: &CheckFactPlan,
    out: &mut PreparedPostgresFacts,
) {
    // Invalid globs are diagnosed when each rule validates/selects its paths.
    let schema_paths = crate::codebase::postgres::postgres_sql_paths(
        root,
        files,
        &crate::codebase::postgres::PostgresSchemaOptions {
            sql_include: plan.postgres_sql_include.clone(),
        },
    )
    .unwrap_or_default();
    let schema_set = crate::codebase::check_facts::PathMembership::new(&schema_paths);
    let mut paths = schema_paths.clone();
    paths.extend(write_sql_paths(root, files, plan));
    paths.sort();
    paths.dedup();
    let rows: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let source = sources.read_path(path).map_err(|error| {
                Arc::new(PreparationError {
                    message: Arc::from(format!("failed to collect PostgreSQL facts: {error}")),
                    source_kind: Some(error.kind()),
                })
            });
            let parsed = source.as_ref().map(|source| {
                let prepared = crate::codebase::postgres::parse::PreparedSql::new(source);
                let parsed = prepared.parse();
                let failed = parsed.is_err();
                (
                    parsed.unwrap_or_else(|_| {
                        crate::codebase::postgres::parse::parse_postgres_sql_lenient_with_sources(
                            source,
                            prepared.normalized(),
                        )
                        .into_iter()
                        .map(|located| located.statement)
                        .collect()
                    }),
                    failed,
                    prepared,
                )
            });
            let schema = (plan.postgres_schema && schema_set.contains(path)).then(|| {
                parsed
                    .as_ref()
                    .map(|(statements, _, _)| {
                        let mut value = crate::codebase::postgres::migration::extract_from_parsed(
                            source.as_ref().unwrap(),
                            statements,
                        );
                        value.path = path.clone();
                        Arc::new(value)
                    })
                    .map_err(|error| Arc::clone(error))
            });
            let statements = plan.postgres_dml.then(|| {
                parsed
                    .as_ref()
                    .map(|(statements, failed, prepared)| {
                        let mut value = crate::codebase::postgres::statements::extract_from_parsed_with_recovered_placeholders(
                            source.as_ref().unwrap(),
                            prepared,
                            statements,
                            *failed,
                            plan.postgres_bounds,
                            None,
                        );
                        value.path = path.clone();
                        Arc::new(vec![value])
                    })
                    .map_err(|error| Arc::clone(error))
            });
            (path.clone(), schema, statements)
        })
        .collect();
    for (path, schema, statements) in rows {
        if let Some(schema) = schema {
            out.schema.insert(path.clone(), schema);
        }
        if let Some(statements) = statements {
            out.statements.insert(path, statements);
        }
    }
}
