//! Native SQL projections from one prepared parse per selected source.
use super::*;
use crate::codebase::postgres::parse::PreparedSql;
use rayon::prelude::*;
use sqlparser::keywords::Keyword;

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
    let write_paths = write_sql_paths(root, files, plan);
    let dml_write_set = (plan.postgres_dml
        && !plan.postgres_explain_write_rollback_paths.is_empty())
    .then(|| crate::codebase::check_facts::PathMembership::new(&write_paths));
    paths.extend(write_paths.iter().cloned());
    paths.extend(plan.postgres_explain_write_rollback_paths.iter().cloned());
    paths.sort();
    paths.dedup();
    let rows: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let source = sources.read_source_path(path).map_err(|error| {
                Arc::new(PreparationError {
                    message: Arc::from(format!("failed to collect PostgreSQL facts: {error}")),
                    source_kind: Some(error.kind()),
                })
            });
            let schema_selected = plan.postgres_schema && schema_set.contains(path);
            let dml_selected = plan.postgres_dml
                && dml_write_set.as_ref().is_none_or(|writes| schema_set.contains(path) || writes.contains(path));
            let explain_selected = plan.postgres_explain_write_rollback_paths.contains(path);
            let prepared = source.as_ref().map(|source| (source, PreparedSql::new(source)));
            let explain_absent = explain_selected
                && prepared.as_ref().is_ok_and(|(_, sql)| sql.keyword_is_absent(Keyword::EXPLAIN));
            if explain_absent && !schema_selected && !dml_selected {
                return (path.clone(), None, None, Some(Ok(Arc::default())));
            }
            let mut write_plan_locations = (explain_selected && !explain_absent).then(Vec::new);
            let parsed = prepared.map(|(source, prepared)| {
                let parsed = prepared.parse_policy_with_locations(write_plan_locations.as_mut());
                let failed = parsed.is_err();
                let (statements, functions, failures, statement_sources, grouping_complete) = match parsed {
                    Ok(statements) => (statements, Vec::new(), Vec::new(), Vec::new(), None),
                    Err(_) if !schema_selected && !dml_selected => {
                        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), None)
                    }
                    Err(_) => {
                        let recovered = crate::codebase::postgres::parse::parse_postgres_sql_lenient_with_sources(
                            source,
                            prepared.normalized(),
                        );
                        let failures = crate::codebase::postgres::migration::recovery_failures(&recovered);
                        let (mut located, functions) = crate::codebase::postgres::parse::partition_function_sources(recovered);
                        prepared.adopt_recovered_arbiters(&mut located);
                        let grouping_complete = plan.postgres_grouping.then(|| located.iter().map(|statement| statement.complete).collect::<Vec<_>>());
                        {
                            let sources: Vec<_> = located.iter().map(|located| located.source.clone().or_else(|| located.chunk_tokens.clone())).collect();
                            (located.into_iter().map(|located| located.statement).collect(), functions, failures, sources, grouping_complete)
                        }
                    }
                };
                (statements, functions, failures, failed, prepared, statement_sources, grouping_complete)
            });
            let write_plans = write_plan_locations.map(|locations| {
                parsed
                    .as_ref()
                    .map(|(statements, _, _, failed, _, _, _)| {
                        let plans = if *failed {
                            explain_write_rollback::unparseable()
                        } else if locations.len() != statements.len() {
                            explain_write_rollback::partial_program()
                        } else {
                            explain_write_rollback::collect(statements, &locations)
                        };
                        Arc::new(plans)
                    })
                    .map_err(|error| Arc::clone(*error))
            }).or_else(|| explain_absent.then(|| Ok(Arc::default())));
            let schema = schema_selected.then(|| {
                parsed
                    .as_ref()
                    .map(|(statements, functions, failures, _, prepared, statement_sources, _)| {
                        let mut functions = functions.clone();
                        functions.extend_from_slice(&prepared.functions());
                        let tokens: Vec<_> = statements.iter().enumerate().map(|(index, _)| statement_sources.get(index).and_then(|source| source.as_deref()).unwrap_or_else(|| prepared.tokens())).collect();
                        let mut value = crate::codebase::postgres::migration::extract_from_parsed_with_failures(
                            source.as_ref().unwrap(),
                            statements,
                            &functions,
                            failures,
                            &tokens,
                        );
                        value.set_path(path.clone());
                        Arc::new(value)
                    })
                    .map_err(|error| Arc::clone(*error))
            });
            let statements = dml_selected.then(|| {
                parsed
                    .as_ref()
                    .map(|(statements, functions, failures, failed, prepared, statement_sources, grouping_complete)| {
                        let mut value = crate::codebase::postgres::statements::extract_from_parsed_with_recovered_placeholders(
                            source.as_ref().unwrap(),
                            prepared,
                            statements,
                            *failed,
                            plan.postgres_bounds,
                            None,
                            crate::codebase::postgres::statements::StatementPolicySources {
                                bind_values: Default::default(),
                                tokens: Some(statement_sources),
                                schema: schema.as_ref().and_then(|entry| entry.as_ref().ok()).map(std::sync::Arc::as_ref),
                                functions,
                                collect_lock_inventory: plan.postgres_lock_inventory,
                                collect_json_writes: plan.postgres_json_writes,
                                collect_duplicate_keyed_lookups: plan.postgres_duplicate_keyed_lookups,
                                collect_grouping: plan.postgres_grouping,
                                grouping_complete: grouping_complete.as_deref(),
                                collect_index_probes: plan.postgres_index_probes,
                                collect_body_bounds: plan.postgres_body_bounds,
                                share_lock_writes: false,
                                redundant_prelocks: false,
                            },
                        );
                        value.parse_failures = failures.iter()
                            .filter(|failure| failure.statement_unanalyzable).cloned().collect();
                        value.path = path.clone();
                        Arc::new(vec![value])
                    })
                    .map_err(|error| Arc::clone(*error))
            });
            (path.clone(), schema, statements, write_plans)
        })
        .collect();
    for (path, schema, statements, write_plans) in rows {
        if let Some(write_plans) = write_plans {
            out.explain_write_rollback.insert(path.clone(), write_plans);
        }
        if let Some(schema) = schema {
            out.schema.insert(path.clone(), schema);
        }
        if let Some(statements) = statements {
            out.statements.insert(path, statements);
        }
    }
}
