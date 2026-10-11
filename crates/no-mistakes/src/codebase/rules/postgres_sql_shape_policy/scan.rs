use super::{iteration, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::postgres_sql_paths;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};
mod diagnostic;
mod fragments;
pub(super) use diagnostic::finding;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &CheckFactMap,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
    let mut fragment_findings = fragments::Findings::default();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = facts
            .embedded_sql(path, &opts.embedded)
            .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for shape policy",
                    opts.shapes.unanalyzable_target(),
                ));
            }
        }
        for (index, fragment) in file.fragments.iter().enumerate() {
            if fragment.sql_text.is_none()
                && file.fragment_variants.get(index).is_none_or(Vec::is_empty)
                && opts.fail_unanalyzable
            {
                findings.push(finding(
                    &rel,
                    fragment.line.max(1) as usize,
                    "builder SQL is not statically recoverable for shape policy",
                    opts.shapes.unanalyzable_target(),
                ));
            }
        }
        let source = crate::codebase::rules::read_source(sources, path);
        for fragment in facts.postgres_fragments(path, &opts.embedded)? {
            let statements = &fragment.statements;
            let line_at =
                |line: usize| fragment.line.saturating_add(line as u32).saturating_sub(1) as usize;
            for (index, finding) in super::functions::located_findings(
                &rel,
                statements,
                &opts.shapes,
                &opts.banned_functions,
                line_at,
            ) {
                fragment_findings.push(
                    fragment,
                    crate::codebase::postgres::statements::SqlFactSite::Function(index),
                    source.as_deref(),
                    finding,
                );
            }
            if opts.fail_unanalyzable && statements.parse_failed {
                fragment_findings.push(
                    fragment,
                    crate::codebase::postgres::statements::SqlFactSite::Origin,
                    source.as_deref(),
                    finding(
                        &rel,
                        fragment.line.max(1) as usize,
                        "builder SQL is not statically recoverable for shape policy",
                        opts.shapes.unanalyzable_target(),
                    ),
                );
                continue;
            }
            for (select_index, select) in statements.selects.iter().enumerate() {
                for (site, finding) in super::select_findings::located_findings(
                    &rel,
                    select,
                    &opts.shapes,
                    select_index,
                    line_at,
                ) {
                    fragment_findings.push(fragment, site, source.as_deref(), finding);
                }
            }
            for (site, finding) in iteration::located_findings(
                &rel,
                statements,
                &opts.shapes,
                &opts.iteration,
                line_at,
            ) {
                fragment_findings.push(fragment, site, source.as_deref(), finding);
            }
        }
    }
    let sql_paths = postgres_sql_paths(root, files, &opts.schema)?;
    let projections = sql_paths.iter().map(|path| (path, None)).chain(
        files
            .iter()
            .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
            .map(|path| (path, Some(&opts.embedded))),
    );
    for (path, profile) in projections {
        for file in facts.postgres_statements(path, profile)? {
            let rel = relative_slash_path(root, &file.path);
            let source = crate::codebase::rules::read_source(sources, path);
            for (index, finding) in super::functions::located_findings(
                &rel,
                file,
                &opts.shapes,
                &opts.banned_functions,
                |line| line.max(1),
            ) {
                fragment_findings.push_complete(
                    &mut findings,
                    file,
                    crate::codebase::postgres::statements::SqlFactSite::Function(index),
                    source.as_deref(),
                    finding,
                    &mut dedup,
                );
            }
            if opts.fail_unanalyzable && file.parse_failed {
                fragment_findings.push_complete(
                    &mut findings,
                    file,
                    crate::codebase::postgres::statements::SqlFactSite::Origin,
                    source.as_deref(),
                    finding(
                        &rel,
                        file.origin_line.max(1),
                        "SQL could not be analyzed for shape policy",
                        opts.shapes.unanalyzable_target(),
                    ),
                    &mut dedup,
                );
                continue;
            }
            for (select_index, select) in file.selects.iter().enumerate() {
                for (site, finding) in super::select_findings::located_findings(
                    &rel,
                    select,
                    &opts.shapes,
                    select_index,
                    |line| line.max(1),
                ) {
                    fragment_findings.push_complete(
                        &mut findings,
                        file,
                        site,
                        source.as_deref(),
                        finding,
                        &mut dedup,
                    );
                }
            }
            for (site, finding) in
                iteration::located_findings(&rel, file, &opts.shapes, &opts.iteration, |line| {
                    line.max(1)
                })
            {
                fragment_findings.push_complete(
                    &mut findings,
                    file,
                    site,
                    source.as_deref(),
                    finding,
                    &mut dedup,
                );
            }
        }
    }
    fragment_findings.extend(&mut findings);
    findings.sort();
    Ok(findings)
}
