use super::*;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub(crate) struct FileRuleInputs<'a> {
    pub root: &'a Path,
    pub config: &'a NoMistakesConfig,
    pub files: &'a [PathBuf],
    pub sources: &'a SourceStore,
    pub defer_suppression: bool,
}

#[derive(Default, Debug)]
pub(crate) struct ClassifiedInputFindings {
    pub(crate) findings: Vec<RuleFinding>,
    pub(crate) uncertainty: Vec<RuleFinding>,
}

pub(crate) fn classified_applications<O: DeserializeOwned + Default>(
    inputs: FileRuleInputs<'_>,
    rule_id: &str,
    scan: impl Fn(&O, &[PathBuf], &[PathBuf], &[PathBuf]) -> Result<ClassifiedInputFindings> + Sync,
) -> Result<AnalysisDiagnostics> {
    let outputs: Result<Vec<_>> = inputs
        .config
        .rule_applications(rule_id)
        .into_par_iter()
        .map(|rule| {
            let options: O = rule.try_rule_options()?;
            let (scope, roots) = super::super::selected_files::rule_scope_files_with_roots(
                inputs.root,
                inputs.config,
                rule,
                inputs.files,
            );
            let files = super::super::path_filter::filter_rule_files(
                inputs.root,
                inputs.config,
                rule,
                &scope,
            )?;
            let ClassifiedInputFindings {
                findings,
                uncertainty,
            } = scan(&options, &files, &scope, &roots)?;
            let mut diagnostics = application(
                inputs.root,
                inputs.sources,
                inputs.config,
                rule,
                findings,
                inputs.defer_suppression,
                InputUncertainty::Read,
            )?;
            if !uncertainty.is_empty() {
                diagnostics.append(application(
                    inputs.root,
                    inputs.sources,
                    inputs.config,
                    rule,
                    uncertainty,
                    inputs.defer_suppression,
                    InputUncertainty::ReadOrParse,
                )?);
            }
            Ok(diagnostics)
        })
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for output in outputs? {
        diagnostics.append(output);
    }
    Ok(sorted(diagnostics))
}
