mod analysis;
mod substitute;

use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::{postgres_sql_paths, EmbeddedSqlKind};
use crate::codebase::rules::postgres_lock_ordering::directive::has_safe_directive;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const OPAQUE_SQL_MESSAGE: &str =
    "keep executor SQL statically recoverable so canonical ON CONFLICT ordering can be checked";
const DYNAMIC_INSERT_MESSAGE: &str =
    "keep dynamic INSERT SQL statically parseable so canonical ON CONFLICT ordering can be checked";

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let catalog = facts.postgres_schema_catalog(&opts.schema_catalog_path)?;
    let mut findings = Vec::new();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = facts.embedded_sql(path, &opts.embedded).with_context(|| {
            format!(
                "{RULE_ID} failed to collect embedded SQL facts from prepared analysis for {}",
                path.display()
            )
        })?;
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path).unwrap_or_default();
        for call in &file.calls {
            if has_safe_directive(
                &source,
                call.line,
                call.sql_text.as_deref().unwrap_or_default(),
                &opts.safe_directive,
            ) {
                continue;
            }
            let sql = call.sql_text.as_deref();
            if call.kind == EmbeddedSqlKind::Dynamic {
                if opts.fail_unanalyzable && sql.is_none_or(analysis::contains_insert) {
                    findings.push(unanalyzable_sql(
                        &rel,
                        call.line,
                        if sql.is_none() {
                            OPAQUE_SQL_MESSAGE
                        } else {
                            DYNAMIC_INSERT_MESSAGE
                        },
                    ));
                }
                continue;
            }
            let Some(sql) = sql else {
                if opts.fail_unanalyzable {
                    findings.push(unanalyzable_sql(&rel, call.line, OPAQUE_SQL_MESSAGE));
                }
                continue;
            };
            findings.extend(analysis::findings_for_sql(
                &rel,
                call.line as usize,
                sql,
                catalog,
                opts.fail_unanalyzable,
            ));
        }
    }
    if let Some(sql_sources) = &opts.sql_sources {
        for path in postgres_sql_paths(root, files, sql_sources)? {
            let rel = relative_slash_path(root, &path);
            let source = crate::codebase::rules::read_source(sources, &path).unwrap_or_default();
            for (line, statement) in analysis::sql_statements(&source) {
                if has_safe_directive(statement, 1, statement, &opts.safe_directive) {
                    continue;
                }
                findings.extend(analysis::findings_for_sql(
                    &rel,
                    line as usize,
                    statement,
                    catalog,
                    opts.fail_unanalyzable,
                ));
            }
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn unanalyzable_sql(rel: &str, line: u32, message: &'static str) -> RuleFinding {
    analysis::finding(rel, line as usize, "unanalyzable-sql", message)
}
