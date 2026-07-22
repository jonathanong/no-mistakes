use super::{analysis, config, enforce, project_config, resolve, runner_config, sort_findings};
use crate::config::v2::load_v2_config_from_visible;
use anyhow::Result;
use std::path::Path;

pub(super) fn check(
    root: &Path,
    config_path: Option<&Path>,
) -> Result<Vec<super::IntegrationFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let config = load_v2_config_from_visible(root, config_path, &visible_paths)?;
    config::validate_config(&config)?;
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    let runner_configs = runner_config::prepare(root, &config, &visible_paths, &tsconfig);
    let parsed_runner_configs = runner_configs.parse_all()?;
    let suites = config::configured_suites_from_runner_configs(
        root,
        &config,
        &runner_configs,
        &parsed_runner_configs,
    )?;
    if suites.is_empty() {
        return Ok(Vec::new());
    }

    let files = crate::codebase::ts_source::discover_source_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &visible_paths,
    );
    let runner_analyses = parsed_runner_configs.analyses_for(&files);
    let analyses = analysis::analyze_files_with_seed(&files, runner_analyses)?;
    let function_index = resolve::build_function_index(&analyses);
    let export_index = resolve::build_export_index(&analyses);
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(analyses.keys().cloned());
    let visible_files = analyses.keys().cloned().collect();
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let import_resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        &tsconfig,
        Some(&visible_files),
        &session,
    );
    let resolver = resolve::ImportResolution {
        analyses: &analyses,
        export_index: &export_index,
        resolver: &import_resolver,
        remapper: &remapper,
    };

    let mut findings = Vec::new();
    for suite in &suites {
        let include = project_config::build_globset(&suite.include)?;
        let exclude = project_config::build_globset(&suite.exclude)?;
        for (file, file_analysis) in &analyses {
            let rel = crate::codebase::ts_source::relative_slash_path(root, file);
            if !include.is_match(&rel) || exclude.is_match(&rel) {
                continue;
            }
            for test in &file_analysis.tests {
                let integrations =
                    resolve::resolved_integrations(&test.function_key, &function_index, &resolver);
                findings.extend(enforce::enforce_policy(root, suite, test, &integrations));
            }
        }
    }
    sort_findings(&mut findings);
    Ok(findings)
}
