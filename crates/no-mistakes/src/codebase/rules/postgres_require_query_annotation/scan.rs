use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::sql_requires_query_annotation;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<crate::codebase::rules::PostgresFindings> {
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = facts
            .embedded_sql(path, &opts.embedded)
            .with_context(|| format!("{RULE_ID} failed to collect embedded SQL facts"))?;
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path).unwrap_or_default();
        let statements = file
            .calls
            .iter()
            .any(|call| !call.variants.is_empty())
            .then(|| facts.postgres_statements(path, Some(&opts.embedded)))
            .transpose()?
            .map(crate::codebase::rules::index_sql_variants)
            .unwrap_or_default();
        for (call_index, (call, span)) in file.calls.iter().zip(&file.call_spans).enumerate() {
            if !call.variants.is_empty() {
                for (variant_index, version) in call.statement_calls().enumerate() {
                    let statement = statements
                        .get(&(call_index, variant_index))
                        .copied()
                        .ok_or_else(|| anyhow::anyhow!("prepared SQL variant facts are missing"))?;
                    for finding in findings_for_call(&rel, &version) {
                        dedup.push(
                            &mut findings,
                            statement,
                            crate::codebase::postgres::statements::SqlFactSite::Annotation,
                            Some(&source),
                            finding,
                        );
                    }
                }
                continue;
            }
            let prefix = facts.query_annotation_prefix(path, &opts.embedded, *span);
            let sql = match prefix {
                Some(value) => value.as_deref(),
                None => call.sql_text.as_deref(),
            };
            if let Some(sql) = sql {
                let mut projected = call.clone();
                projected.sql_text = Some(sql.to_string());
                findings.extend(findings_for_call(&rel, &projected));
            } else if opts.unanalyzable_sql == super::UnanalyzableSql::Report {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(), file: rel.clone(), line: call.line as usize,
                    message: format!("{rel}:{}: query annotation cannot be verified because the leading SQL is unanalyzable; make the prefix static or add a leading /* name */ annotation", call.line),
                    import: None, target: Some("annotation".to_string()),
                });
            }
        }
    }
    findings.sort();
    Ok(findings)
}

pub(super) fn findings_for_call(
    file: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
) -> Vec<RuleFinding> {
    let Some(sql) = call.sql_text.as_deref() else {
        return Vec::new();
    };
    if !sql_requires_query_annotation(sql) {
        return Vec::new();
    }
    vec![RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: call.line as usize,
        message: format!(
            "{file}:{}: query execution must start with a /* name */ annotation",
            call.line
        ),
        import: None,
        target: Some("annotation".to_string()),
    }]
}
