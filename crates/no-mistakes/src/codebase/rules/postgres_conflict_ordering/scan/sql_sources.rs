//! Opt-in standalone SQL sources share the owning request's prepared source store.
use super::{analysis, CompiledOptions};
use crate::codebase::postgres::{postgres_sql_paths, SchemaCatalog};
use crate::codebase::rules::postgres_lock_ordering::directive::has_safe_directive;
use crate::codebase::rules::PostgresFindings;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    catalog: &SchemaCatalog,
    findings: &mut PostgresFindings,
) -> Result<()> {
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
    Ok(())
}
