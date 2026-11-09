use super::*;
mod column;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::rules::{PostgresFindings, VariantFindingDedup};
pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
) -> Result<PostgresFindings> {
    let catalog = facts.postgres_schema_catalog(&opts.schema_catalog_path)?;
    let results: Vec<_> = files
        .par_iter()
        .map(|path| scan_file(root, opts, path, sources, facts, catalog))
        .collect();
    let findings: Result<Vec<_>> = results.into_iter().collect();
    Ok(findings?.into_iter().flatten().collect())
}
fn scan_file(
    root: &Path,
    opts: &CompiledOptions,
    path: &Path,
    sources: &SourceStore,
    facts: &CheckFactMap,
    catalog: &crate::codebase::postgres::SchemaCatalog,
) -> Result<PostgresFindings> {
    let mut findings = PostgresFindings::default();
    let mut dedup = VariantFindingDedup::for_root(root, sources);
    let file = relative_slash_path(root, path);
    let profile = (!file.ends_with(".sql")).then_some(&opts.embedded);
    let statements = match facts.postgres_statements(path, profile) {
        Ok(value) => value,
        Err(_) if profile.is_none() && facts.postgres.sql_source_not_found(path) => {
            return Ok(findings)
        }
        Err(error) => return Err(error),
    };
    let source = sources.read_path(path)?;
    for statement in statements {
        for (query_index, query) in statement.grouping_queries.iter().enumerate() {
            for finding in analyze::query(&query.query, catalog, &query.parameter_sites) {
                let Some(site) = query.sites.iter().position(|site| *site == finding.site) else {
                    continue;
                };
                let line = finding.site.0.max(1);
                let site = SqlFactSite::Grouping(query_index, site);
                let column = column::label(
                    statement,
                    site.clone(),
                    finding.site.1,
                    path,
                    &source,
                    sources,
                )?;
                dedup.push_merged(
                    &mut findings,
                    statement,
                    site,
                    Some(&source),
                    RuleFinding {
                        causes: Vec::new(),
                        rule: RULE_ID.into(),
                        file: file.clone(),
                        line,
                        message: format!(
                            "{file}:{line}: [{}] {} ({column})",
                            finding.code, finding.message
                        ),
                        import: None,
                        target: Some(finding.code.into()),
                    },
                );
            }
        }
    }
    Ok(findings)
}
