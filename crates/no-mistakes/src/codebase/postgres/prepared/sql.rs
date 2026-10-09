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
                let (statements, functions) = match parsed {
                    Ok(statements) => (statements, Vec::new()),
                    Err(_) => {
                        let (located, functions) = crate::codebase::postgres::parse::partition_function_sources(crate::codebase::postgres::parse::parse_postgres_sql_lenient_with_sources(
                            source,
                            prepared.normalized(),
                        ));
                        (located.into_iter().map(|located| located.statement).collect(), functions)
                    }
                };
                (statements, functions, failed, prepared)
            });
            let schema = (plan.postgres_schema && schema_set.contains(path)).then(|| {
                parsed
                    .as_ref()
                    .map(|(statements, functions, _, prepared)| {
                        let mut functions = functions.clone();
                        functions.extend_from_slice(&prepared.functions());
                        let mut value = crate::codebase::postgres::migration::extract_from_parsed(
                            source.as_ref().unwrap(),
                            statements,
                            &functions,
                        );
                        value.path = path.clone();
                        Arc::new(value)
                    })
                    .map_err(|error| Arc::clone(error))
            });
            let statements = plan.postgres_dml.then(|| {
                parsed
                    .as_ref()
                    .map(|(statements, functions, failed, prepared)| {
                        let mut value = crate::codebase::postgres::statements::extract_from_parsed_with_recovered_placeholders(
                            source.as_ref().unwrap(),
                            prepared,
                            statements,
                            *failed,
                            plan.postgres_bounds,
                            None,
                            crate::codebase::postgres::statements::StatementPolicySources {
                                schema: schema.as_ref().and_then(|entry| entry.as_ref().ok()).map(std::sync::Arc::as_ref),
                                functions,
                            },
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
