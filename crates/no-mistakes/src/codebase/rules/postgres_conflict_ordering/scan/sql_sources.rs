//! Opt-in standalone SQL sources share the owning request's prepared source store.
use super::{analysis, CompiledOptions};
use crate::codebase::postgres::{postgres_sql_paths, SchemaCatalog};
use crate::codebase::rules::postgres_lock_ordering::directive::comment_contains_directive;
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
    facts: &dyn crate::codebase::check_facts::SqlFactContext,
) -> Result<()> {
    if let Some(sql_sources) = &opts.sql_sources {
        for path in postgres_sql_paths(root, files, sql_sources)? {
            let rel = relative_slash_path(root, &path);
            let source = crate::codebase::rules::read_source(sources, &path).unwrap_or_default();
            let advice = opts
                .advise_unanalyzable
                .then(|| crate::codebase::rules::native_sql_advice::NativeSqlAdvice::new(&source));
            for (line, statement) in analysis::sql_statements(&source) {
                if comment_contains_directive(statement, &opts.safe_directive) {
                    continue;
                }
                let checked = analysis::findings_for_sql(
                    &rel,
                    line as usize,
                    statement,
                    catalog,
                    opts.fail_unanalyzable || opts.advise_unanalyzable,
                );
                for finding in checked {
                    if let (Some(advice), Some("unanalyzable-sql")) =
                        (&advice, finding.target.as_deref())
                    {
                        advice.record(
                            facts,
                            super::RULE_ID,
                            &opts.embedded,
                            &rel,
                            finding.line,
                            "SQL could not be analyzed for canonical conflict ordering",
                        );
                    } else {
                        findings.push(finding);
                    }
                }
            }
        }
    }
    Ok(())
}
