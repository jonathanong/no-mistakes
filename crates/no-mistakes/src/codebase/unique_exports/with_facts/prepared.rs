use super::{analyze_project_roots_with_facts, ProjectRootsAnalysis};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::config::Config;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::unique_exports::{UniqueExportFinding, RULE_ID};
use anyhow::Result;
use std::path::Path;

#[doc(hidden)]
pub fn analyze_project_with_config_and_facts(
    root: &Path,
    config: &Config,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<UniqueExportFinding>> {
    let session = AnalysisSession::new(crate::diagnostics::current());
    analyze_project_with_optional_prepared_facts(
        root,
        config,
        tsconfig_path,
        None,
        shared,
        None,
        &session,
    )
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_facts(
    root: &Path,
    config: &Config,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &CheckFactMap,
) -> Result<Vec<UniqueExportFinding>> {
    let session = AnalysisSession::new(crate::diagnostics::current());
    analyze_project_with_optional_prepared_facts(
        root,
        config,
        None,
        Some(tsconfig),
        shared,
        None,
        &session,
    )
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_and_inferred(
    root: &Path,
    config: &Config,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
) -> Result<Vec<UniqueExportFinding>> {
    let session = AnalysisSession::new(crate::diagnostics::current());
    analyze_project_with_prepared_facts_and_inferred_and_session(
        root,
        config,
        tsconfig,
        shared,
        inferred_roots,
        &session,
    )
}

#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_and_inferred_and_session(
    root: &Path,
    config: &Config,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
    session: &AnalysisSession,
) -> Result<Vec<UniqueExportFinding>> {
    analyze_project_with_optional_prepared_facts(
        root,
        config,
        None,
        Some(tsconfig),
        shared,
        Some(inferred_roots),
        session,
    )
}

fn analyze_project_with_optional_prepared_facts(
    root: &Path,
    config: &Config,
    tsconfig_path: Option<&Path>,
    prepared_tsconfig: Option<&crate::codebase::ts_resolver::TsConfig>,
    shared: &CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    session: &AnalysisSession,
) -> Result<Vec<UniqueExportFinding>> {
    let normalized_root = normalize_path(root);
    let root = normalized_root.as_path();
    let applications = config.rule_applications_for(RULE_ID);
    if !applications.is_empty() {
        let mut findings = Vec::new();
        for application in applications {
            let project_roots = match inferred_roots {
                Some(inferred_roots) => config.project_roots_for_rule_application_with_inferred(
                    root,
                    application,
                    inferred_roots,
                ),
                None => config.project_roots_for_rule_application(root, application),
            }
            .into_iter()
            .map(|path| normalize_path(&path))
            .collect::<Vec<_>>();
            let options = application.rule_options();
            findings.extend(analyze_project_roots_with_facts(ProjectRootsAnalysis {
                session,
                root,
                application_filter: Some((config, application)),
                tsconfig_path,
                prepared_tsconfig,
                shared,
                project_roots,
                options,
                inferred_roots,
            })?);
        }
        findings.sort();
        findings.dedup();
        return Ok(findings);
    }
    let project_roots = match inferred_roots {
        Some(inferred_roots) => {
            config.project_roots_for_rule_with_inferred(root, RULE_ID, inferred_roots)
        }
        None => config.project_roots_for_rule(root, RULE_ID),
    }
    .into_iter()
    .map(|path| normalize_path(&path))
    .collect::<Vec<_>>();
    analyze_project_roots_with_facts(ProjectRootsAnalysis {
        session,
        root,
        application_filter: None,
        tsconfig_path,
        prepared_tsconfig,
        shared,
        project_roots,
        options: config.rule_options(RULE_ID),
        inferred_roots,
    })
}
