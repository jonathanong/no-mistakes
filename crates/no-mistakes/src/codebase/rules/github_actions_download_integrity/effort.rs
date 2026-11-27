use super::*;
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics,
    file_diagnostics::{self, InputUncertainty},
    github_actions_workflow_files::{parse_yaml, read_workflow_source},
    path_filter::GlobMatcher,
};
use crate::codebase::ts_source::{
    discover_files_from_visible, relative_slash_path, SourceStore, VisiblePathSnapshot,
};
use crate::config::v2::{Effort, NoMistakesConfig};
use no_mistakes_rule_kernel::paths::normalize_discovery_path;
use rayon::prelude::*;
use std::sync::Arc;

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let snapshot = VisiblePathSnapshot::new(root);
    let files = discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &snapshot.paths_for(root),
    );
    let sources = snapshot.source_store_for(root);
    check_with_snapshot(root, config, &files, &sources, &snapshot)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let snapshot = VisiblePathSnapshot::from_paths(root, files);
    let sources = snapshot.source_store_for(root);
    check_with_snapshot(root, config, files, &sources, &snapshot)
}

pub(crate) fn check_with_snapshot(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    snapshot: &VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    Ok(diagnostics_with_files_and_sources(root, config, files, sources, snapshot, true)?.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    snapshot: &VisiblePathSnapshot,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let (files, roots) =
            super::super::selected_rule_files_with_roots(root, config, rule, files)?;
        let analysis = config.effective_rule_analysis(rule)?;
        let hosts = compile_hosts(&options)?;
        let policy = runs::Policy {
            hosts: &hosts,
            helpers: &options.download_helpers,
            low: (analysis.effort == Effort::Low).then_some(analysis.unanalyzable),
            strict: options.strict_sequencing,
            skip_scope: options.skip_annotation_scope,
        };
        let includes = GlobMatcher::new(
            &options.shell_source_include,
            &format!("{RULE_ID} shellSourceInclude"),
        )?;
        let indexes: Vec<_> = roots
            .into_iter()
            .filter(|target| !includes.is_empty() && snapshot.git_index_available_for(target))
            .map(|target| snapshot.git_index_paths_for(&target))
            .collect();
        let chunks: Vec<_> = files
            .par_iter()
            .filter_map(|path| {
                let rel = relative_slash_path(root, path);
                let kind = gha_kind(&rel);
                if kind.is_none()
                    && !(includes.is_match(&rel) && indexed(&indexes, path))
                {
                    return None;
                }
                if let Some(action) = kind {
                    return Some(read_workflow_source(root, path, sources, RULE_ID).and_then(
                        |content| {
                            parse_yaml(root, path, sources, &content, RULE_ID, true).map(|value| {
                                runs::check_workflow(&rel, &content, &value, action, &policy)
                            })
                        },
                    ));
                }
                let content = super::super::read_language_source(sources, path)?;
                Some(Ok(runs::check_shell(&rel, &content, &policy)))
            })
            .collect();
        let mut input_failures = Vec::new();
        for chunk in chunks {
            match chunk {
                Ok(output) => diagnostics.append(output),
                Err(finding) => input_failures.push(*finding),
            }
        }
        diagnostics.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            input_failures,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    if !defer_suppression {
        super::super::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        super::super::suppress_rule_findings_with_sources(
            root,
            &mut diagnostics.advisories,
            sources,
        );
    }
    super::super::sort_findings(&mut diagnostics.findings);
    super::super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

/// Index entries are normalized, but inventory paths can mix separators on
/// Windows (`dir\scripts/x.sh`), so normalize before the exact probe.
pub(super) fn indexed(indexes: &[Arc<Vec<PathBuf>>], path: &Path) -> bool {
    let path = normalize_discovery_path(path);
    indexes.iter().any(|index| {
        index
            .binary_search_by(|entry| entry.as_os_str().cmp(path.as_os_str()))
            .is_ok()
    })
}
