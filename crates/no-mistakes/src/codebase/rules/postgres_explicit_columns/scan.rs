use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use crate::codebase::postgres::SchemaCatalog;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

mod finding;
mod fragment_ast;
mod fragment_parse;
mod fragments;
mod source;
mod verdicts;
use finding::{message, sql_finding, star_findings};
use source::physical_source;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&dyn SqlFactContext>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    // Every entrypoint prepares request-owned SQL facts before reaching this scan.
    let prepared =
        prepared.ok_or_else(|| anyhow::anyhow!("prepared PostgreSQL facts are required"))?;
    let inputs = crate::codebase::rules::collect_prepared_sql_inputs(
        root,
        files,
        &opts.schema,
        &opts.schema_catalog_path,
        &opts.embedded,
        Some(prepared),
    )?;
    let catalog = inputs.catalog;
    let native_statement_count = inputs.native_statement_count;
    let statements = inputs.statements;
    let embedded = inputs.embedded;
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    if prepared.wants_sql_verdicts() {
        verdicts::record(root, opts, &embedded, catalog, prepared)?;
    }
    for file in &embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    &prepared.sql_unanalyzable_message(
                        "executed SQL is not statically recoverable for explicit columns",
                    ),
                    "unanalyzable",
                ));
            }
        }
    }
    for (file_index, file) in statements.into_iter().enumerate() {
        let rel = relative_slash_path(root, &file.path);
        let source = no_mistakes_rule_kernel::source_access::read_language_source(sources, &file.path);
        if opts.advise_unanalyzable && file_index < native_statement_count {
            crate::codebase::rules::native_sql_advice::record_parse_failures(
                prepared,
                RULE_ID,
                &opts.embedded,
                &rel,
                source.as_deref().unwrap_or_default(),
                file,
            );
        }
        if opts.fail_unanalyzable && file.parse_failed {
            for (site, line) in file.parse_failure_sites() {
                dedup.push(
                    &mut findings,
                    file,
                    site,
                    source.as_deref(),
                    sql_finding(
                        &rel,
                        line,
                        "SQL could not be analyzed for explicit columns",
                        "unanalyzable",
                    ),
                );
            }
        }
        let from_variant = file.variant_locations.is_some();
        for (select_index, select) in file.selects.iter().enumerate() {
            for (star_index, star) in select.star_projections.iter().enumerate() {
                let site = SqlFactSite::Star(select_index, star_index);
                let (physical_rel, physical_source) =
                    physical_source(root, sources, file, &site, &rel, source.as_ref());
                let produced = star_findings(
                    &physical_rel,
                    std::slice::from_ref(star),
                    "SELECT *",
                    "reads",
                    opts,
                    catalog,
                );
                if produced.is_empty() && file_index >= native_statement_count {
                    fragments::check_executed_unresolved_star(
                        prepared,
                        opts,
                        catalog,
                        fragments::ExecutedStarContext {
                            rel: &physical_rel,
                            source: physical_source.as_deref(),
                            returning: false,
                            from_variant,
                        },
                        star,
                    );
                }
                for finding in produced {
                    dedup.push(
                        &mut findings,
                        file,
                        site.clone(),
                        physical_source.as_deref(),
                        finding,
                    );
                }
            }
        }
        if opts.check_returning {
            for (star_index, star) in file.returning_stars.iter().enumerate() {
                let site = SqlFactSite::ReturningStar(star_index);
                let (physical_rel, physical_source) =
                    physical_source(root, sources, file, &site, &rel, source.as_ref());
                let produced = star_findings(
                    &physical_rel,
                    std::slice::from_ref(star),
                    "RETURNING *",
                    "returns",
                    opts,
                    catalog,
                );
                if produced.is_empty() && file_index >= native_statement_count {
                    fragments::check_executed_unresolved_star(
                        prepared,
                        opts,
                        catalog,
                        fragments::ExecutedStarContext {
                            rel: &physical_rel,
                            source: physical_source.as_deref(),
                            returning: true,
                            from_variant,
                        },
                        star,
                    );
                }
                for finding in produced {
                    dedup.push(
                        &mut findings,
                        file,
                        site.clone(),
                        physical_source.as_deref(),
                        finding,
                    );
                }
            }
        }
    }
    fragments::scan_embedded_fragments(
        root,
        opts,
        &embedded,
        catalog,
        sources,
        prepared,
        &mut findings,
    );
    findings.sort();
    Ok(findings)
}
