//! Production exports whose syntactic consumers all belong to configured tests.
mod config;
mod nextjs;
mod selector_advisories;
mod selector_gaps;
mod traversal;

use super::analysis_policy::AnalysisDiagnostics;
use super::path_filter::RulePathFilter;
use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::ts_symbols::{ExportKind, FileSymbols};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub const RULE_ID: &str = "production-exports-only-used-by-tests";

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| GraphBuildPlan {
        symbol_import_selectors: true,
        ..Default::default()
    })
}

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> Result<AnalysisDiagnostics> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut diagnostics = AnalysisDiagnostics::default();
    let incomplete = traversal::incomplete_sources(graph, facts);
    for application in config.rule_applications(RULE_ID) {
        let options = config::Selection::new(&root, application)?;
        let filter = RulePathFilter::new(&root, config, application)?;
        let policy = config.effective_rule_analysis(application)?.unanalyzable;
        let files = facts.graph_file_universe();
        let nextjs_roots = nextjs::project_roots(&root, config);
        for (file, line, reason) in &incomplete {
            diagnostics.unanalyzable(policy, uncertain(&root, application, file, *line, reason));
        }
        let chunks: Vec<_> = files
            .par_iter()
            .filter(|file| {
                crate::codebase::dependencies::extract::is_indexable(file)
                    && options.production(file)
                    && filter.is_match(file)
            })
            .map(|file| {
                let mut chunk = AnalysisDiagnostics::default();
                let Some(ts) = facts.get_ts_facts(file) else {
                    return chunk;
                };
                let Some(symbols) = ts
                    .symbols
                    .as_ref()
                    .filter(|_| ts.parse_error.is_none() && ts.operational_error.is_none())
                else {
                    return chunk;
                };
                for export in &symbols.exports {
                    if forwarded(symbols, export) {
                        continue;
                    }
                    let name = if matches!(export.kind, ExportKind::Default) {
                        "default"
                    } else {
                        &export.name
                    };
                    // Next.js reads these names. Other exports on the same file stay in scope.
                    if nextjs::consumed_export(file, &nextjs_roots, name) {
                        continue;
                    }
                    let mut consumers = traversal::consumers(graph, file, name);
                    let undetermined = consumers.settle(&export.kind);
                    if consumers.files.iter().any(|file| !options.test(file)) {
                        continue;
                    }
                    if !consumers.unknown.is_empty() {
                        if let Some((importer, line, reason)) = selector_advisories::for_export(
                            &root,
                            graph,
                            &consumers.unknown,
                            &export.name,
                        ) {
                            chunk.unanalyzable(
                                policy,
                                uncertain(&root, application, &importer, line, &reason),
                            );
                        }
                    } else if !undetermined.is_empty() {
                        let (importer, line, reason) = selector_advisories::undetermined(
                            &root,
                            graph,
                            &undetermined,
                            &export.name,
                        );
                        chunk.unanalyzable(
                            policy,
                            uncertain(&root, application, &importer, line, &reason),
                        );
                    } else if !consumers.files.is_empty()
                        && consumers.files.iter().all(|file| options.test(file))
                    {
                        chunk
                            .findings
                            .push(finding(&root, application, file, export));
                    }
                }
                chunk
            })
            .collect();
        for chunk in chunks {
            diagnostics.append(chunk);
        }
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn forwarded(symbols: &FileSymbols, export: &crate::codebase::ts_symbols::Export) -> bool {
    matches!(export.kind, ExportKind::ReExport { .. })
        || symbols
            .imports
            .iter()
            .any(|import| import.local == export.local.as_deref().unwrap_or(&export.name))
}

fn finding(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    export: &crate::codebase::ts_symbols::Export,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: relative_slash_path(root, file),
        line: export.line.max(1) as usize,
        message: application.message.as_ref().map_or_else(|| format!(
            "production export `{}` is imported only by configured tests or test helpers; move it into a configured test-helper root or remove its export. Production APIs should serve production code",
            export.name,
        ), |message| format!("{message}: `{}`", export.name)),
        import: None,
        target: Some(export.name.clone()),
    }
}

fn uncertain(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    line: usize,
    reason: &str,
) -> RuleFinding {
    let detail = format!("cannot analyze production export importers: {reason}; use explicit named imports and statically analyzable exports");
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: relative_slash_path(root, file),
        line: line.max(1),
        message: application
            .message
            .as_ref()
            .map_or_else(|| detail.clone(), |message| format!("{message}: {detail}")),
        import: None,
        target: Some("unanalyzable".to_owned()),
    }
}

#[cfg(test)]
mod nextjs_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod work_tests;
#[cfg(test)]
mod workspace_exports_tests;
