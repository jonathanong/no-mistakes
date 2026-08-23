use super::{enforce, project_config, resolve, types, IntegrationFinding};
use anyhow::Result;
use std::path::Path;

pub(super) fn fail_on_dropped_files(
    root: &Path,
    suites: &[types::Suite],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<()> {
    let suite_globs = suites
        .iter()
        .map(|suite| {
            Ok((
                project_config::build_globset(&suite.include)?,
                project_config::build_globset(&suite.exclude)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    for (file, facts) in &shared.ts {
        if let Some(error) = &facts.parse_error {
            let disabled_suite_test = facts.source.as_deref().is_some_and(|source| {
                crate::codebase::ts_source::has_disable_file_comment(
                    source,
                    "integration-test-no-mocks",
                )
            }) && suite_globs.iter().any(|(include, exclude)| {
                let relative = crate::codebase::ts_source::relative_slash_path(root, file);
                include.is_match(&relative) && !exclude.is_match(&relative)
            });
            if disabled_suite_test {
                continue;
            }
            anyhow::bail!(
                "failed to parse integration file {}: {error}",
                file.display()
            );
        }
    }
    Ok(())
}

pub(super) fn check_suites(
    root: &Path,
    suites: &[types::Suite],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    analyses: &std::collections::BTreeMap<std::path::PathBuf, types::FileAnalysis>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<Vec<IntegrationFinding>> {
    let visible_files: std::collections::HashSet<std::path::PathBuf> =
        analyses.keys().cloned().collect();
    let import_resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        tsconfig,
        Some(&visible_files),
        session,
    );
    check_suites_with_resolver(root, suites, analyses, &import_resolver)
}

pub(super) fn check_suites_with_resolver<R: crate::codebase::ts_resolver::ImportResolution>(
    root: &Path,
    suites: &[types::Suite],
    analyses: &std::collections::BTreeMap<std::path::PathBuf, types::FileAnalysis>,
    import_resolver: &R,
) -> Result<Vec<IntegrationFinding>> {
    let function_index = resolve::build_function_index(analyses);
    let export_index = resolve::build_export_index(analyses);
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(analyses.keys().cloned());
    let resolver = resolve::ImportResolution {
        analyses,
        export_index: &export_index,
        resolver: import_resolver,
        remapper: &remapper,
    };

    let mut findings = Vec::new();
    for suite in suites {
        let include = project_config::build_globset(&suite.include)?;
        let exclude = project_config::build_globset(&suite.exclude)?;
        for (file, file_analysis) in analyses {
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

pub(crate) fn sort_findings(findings: &mut Vec<IntegrationFinding>) {
    findings.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.framework.cmp(&b.framework))
            .then(a.suite.cmp(&b.suite))
    });
    findings.dedup_by(|a, b| {
        a.framework == b.framework
            && a.suite == b.suite
            && a.file == b.file
            && a.line == b.line
            && a.message == b.message
    });
}
