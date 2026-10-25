use super::evaluate::Columns;
use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::SqlObligationVerdict;
use crate::codebase::rules::PostgresFindings;
use crate::codebase::ts_source::{SourceStore, relative_slash_path};
use crate::config::v2::schema::RuleDef;
use std::path::{Path, PathBuf};
mod cache;
mod evaluation;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
    index: usize,
    rule: &RuleDef,
) -> anyhow::Result<PostgresFindings> {
    let context = facts.for_sql_application_with_advice(index, rule.name.as_deref());
    let sql_files = files
        .iter()
        .filter(|file| file.extension().is_some_and(|ext| ext == "sql"))
        .cloned()
        .collect::<Vec<_>>();
    let mut findings = super::scan::scan(root, opts, &sql_files, sources, &context)?;
    let columns = Columns::new(facts.postgres_schema_catalog(&opts.schema_catalog_path)?);
    for path in files
        .iter()
        .filter(|file| crate::codebase::dependencies::extract::is_indexable(file))
    {
        let rel = relative_slash_path(root, path);
        let file = facts
            .ts
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("{RULE_ID}: prepared facts missing for {rel}"))?;
        let application =
            crate::codebase::postgres::sql_identifier_syntax::helper_recovery::ApplicationKey::new(
                &opts.embedded,
                RULE_ID,
                index,
            );
        let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
            if file.parse_error.is_some() || file.ts.parse_error.is_some() {
                continue;
            }
            anyhow::bail!("{RULE_ID}: prepared low-effort SQL syntax projection missing for {rel}");
        };
        let source = sources.read_path(path)?;
        let literals = cache::literals(file, facts, path, &source);
        for call in calls {
            let suppressed_call = crate::codebase::ts_source::matching_disable_directive(
                &source,
                Some(call.line),
                RULE_ID,
            )
            .is_some();
            let (evaluation, call_findings) = evaluation::evaluate(
                root,
                path,
                &rel,
                &application,
                call,
                literals,
                &columns,
                opts,
                suppressed_call,
            );
            findings.extend(call_findings);
            if !context.wants_sql_verdicts() && !(evaluation.uncertain && opts.advise_unanalyzable)
            {
                continue;
            }
            let verdict = SqlObligationVerdict {
                profile: opts.embedded.clone(),
                rule: RULE_ID.into(),
                application: None,
                subcheck: "uuid-predicates".into(),
                file: rel.clone(),
                span: call.span,
                status: evaluation.status,
                advisory: evaluation.uncertain && opts.advise_unanalyzable,
                evidence: None,
                known_prefix: None,
                helper_lines: Vec::new(),
                residual_reasons: evaluation.reasons,
            };
            if context.wants_sql_verdicts() {
                context.record_sql_verdict(verdict);
            } else {
                context.record_sql_advisory(verdict);
            }
        }
    }
    findings.sort();
    Ok(findings)
}
