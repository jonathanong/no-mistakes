use super::{fragments, PreparationError, PreparedPostgresFacts};
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub(super) fn collect(
    root: &Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
    facts: &CheckFactMap,
    demand: SqlLocationDemand,
    out: &mut PreparedPostgresFacts,
) {
    let observer = crate::diagnostics::current();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        for profile in &plan.embedded_sql_options {
            // Keep extraction broad for imported helpers; only consumer-owned
            // SQL projections follow the union of configured rule scans.
            if plan
                .embedded_sql_rule_ids
                .get(profile)
                .is_some_and(|scans| {
                    !scans.is_empty() && !scans.iter().any(|scan| scan.paths.includes(&rel))
                })
            {
                continue;
            }
            let file = facts.embedded_sql(path, profile);
            if plan.postgres_fragments {
                let entry = file
                    .as_ref()
                    .map(|file| {
                        if let Some(observer) = &observer {
                            observer.increment("postgres.prepared.fragment_projections", 1);
                        }
                        Arc::new(fragments::collect(file))
                    })
                    .map_err(|error| {
                        Arc::new(PreparationError {
                            message: Arc::from(format!(
                                "failed to collect PostgreSQL facts: {error}"
                            )),
                            source_kind: None,
                        })
                    });
                out.fragments.insert((path.clone(), profile.clone()), entry);
            }
            if !plan.postgres_dml && !plan.postgres_variants {
                continue;
            }
            let entry = file
                .map(|file| {
                    if let Some(observer) = &observer {
                        observer.increment("postgres.prepared.statement_projections", 1);
                    }
                    let statements =
                        super::super::collect::dml::embedded_call_facts_with_json_writes(
                            file,
                            plan.postgres_bounds,
                            !plan.postgres_dml,
                            demand,
                            plan.postgres_json_writes,
                        );
                    if let Some(observer) = &observer {
                        observer.increment(
                            "postgres.prepared.statement_records",
                            statements.len() as u64,
                        );
                    }
                    Arc::new(statements)
                })
                .map_err(|error| {
                    Arc::new(PreparationError {
                        message: Arc::from(format!("failed to collect PostgreSQL facts: {error}")),
                        source_kind: None,
                    })
                });
            out.embedded.insert((path.clone(), profile.clone()), entry);
        }
    }
}
