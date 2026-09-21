use super::super::{classify_prepared, ResolveCheckReport, Status};
use super::{batch_report, BatchResolveCheckReport};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_resolver::{ImportResolver, VisiblePathLookup};
use anyhow::Result;
use rayon::prelude::*;

/// Resolve checks for a file closure whose imports were already collected by
/// `analyzeProject`. No source discovery or parsing occurs on this path.
pub(crate) fn batch_report_from_prepared_facts(
    root: &std::path::Path,
    files: impl IntoIterator<Item = std::path::PathBuf>,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
    visible: &dyn VisiblePathLookup,
    source_store: &crate::codebase::ts_source::SourceStore,
    explicit_tsconfig: Option<&crate::codebase::ts_resolver::TsConfig>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<BatchResolveCheckReport> {
    let mut files: Vec<_> = files
        .into_iter()
        .filter(|path| is_indexable(path))
        .collect();
    files.sort();
    files.dedup();
    let visible_cache_key = visible.visible_cache_key();
    let mut results = files
        .par_iter()
        .map(|file| {
            let file_facts = facts
                .get(file)
                .ok_or_else(|| anyhow::anyhow!("missing prepared facts for {}", file.display()))?;
            if let Some(error) = &file_facts.operational_error {
                anyhow::bail!("{error}");
            }
            if file_facts.fatal_parse_error {
                anyhow::bail!(
                    "failed to parse {}: {}",
                    file.display(),
                    file_facts
                        .parse_error
                        .as_deref()
                        .unwrap_or("parser panicked without a diagnostic")
                );
            }
            let tsconfig = match explicit_tsconfig {
                Some(config) => config.clone(),
                None => crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
                    None,
                    file,
                    &visible_cache_key,
                    source_store,
                )
                .unwrap_or_else(|_| crate::codebase::ts_resolver::TsConfig {
                    dir: root.to_path_buf(),
                    paths: Vec::new(),
                    paths_dir: root.to_path_buf(),
                    base_url: None,
                }),
            };
            let resolver = ImportResolver::new_in_session_with_visible_cache_key(
                &tsconfig,
                Some(visible),
                Some(&visible_cache_key),
                session,
            );
            let imports = file_facts
                .imports
                .iter()
                .map(|imp| classify_prepared(imp, file, root, &resolver))
                .collect::<Vec<_>>();
            let unresolved: Vec<String> = imports
                .iter()
                .filter(|row| row.status == Status::Unresolved)
                .map(|row| row.specifier.clone())
                .collect();
            Ok(ResolveCheckReport {
                file: super::super::super::shared::rel_str(file, root),
                all_resolve: unresolved.is_empty(),
                imports,
                unresolved,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    results.sort_by(|left, right| left.file.cmp(&right.file));
    Ok(batch_report(results))
}

#[cfg(test)]
mod tests {
    use super::batch_report_from_prepared_facts;
    use crate::codebase::queries::shared::resolve_targets;
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use std::path::PathBuf;

    #[test]
    fn derived_resolve_check_propagates_prepared_fact_failures() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture");
        let target = resolve_targets(&[PathBuf::from("consumer.ts")], Some(&root), None)
            .unwrap()
            .remove(0);
        let cases = [
            (
                TsFileFacts {
                    operational_error: Some("failed to read consumer.ts".to_string()),
                    ..TsFileFacts::default()
                },
                "failed to read consumer.ts",
            ),
            (
                TsFileFacts {
                    parse_error: Some("parser panicked".to_string()),
                    fatal_parse_error: true,
                    ..TsFileFacts::default()
                },
                "parser panicked",
            ),
            (
                TsFileFacts {
                    fatal_parse_error: true,
                    ..TsFileFacts::default()
                },
                "parser panicked without a diagnostic",
            ),
        ];
        for (file_facts, expected) in cases {
            let facts = TsFactMap::from([(target.abs_file.clone(), file_facts)]);
            let error = batch_report_from_prepared_facts(
                &target.root,
                [target.abs_file.clone()],
                &facts,
                target.visible_files(),
                &target.sources,
                None,
                &target.session,
            )
            .err()
            .expect("prepared fact failure must abort the report");
            assert!(error.to_string().contains(expected), "{error:#}");
        }
    }
}
