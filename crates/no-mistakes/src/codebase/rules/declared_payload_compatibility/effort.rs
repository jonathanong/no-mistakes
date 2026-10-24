use super::*;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::{schema::RuleDef, Effort};

/// Evaluate each contract once with its application's independent effort policy.
#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let documents = document::Documents::default();
    let chunks: Result<Vec<_>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let options: DeclaredPayloadCompatibilityOptions = rule.try_rule_options()?;
            let allowed = super::super::path_filter::filter_rule_files(root, config, rule, files)?;
            let allowed: BTreeSet<_> = allowed
                .iter()
                .map(|path| relative_slash_path(root, path))
                .collect();
            let policy = config.effective_rule_analysis(rule)?;
            let output = options
                .contracts
                .par_iter()
                .map(|contract| {
                    let mut output = AnalysisDiagnostics::default();
                    let result = if policy.effort == Effort::Low {
                        match low::check(root, contract, &allowed, sources, &documents) {
                            Ok(low::Outcome::Proven) => Ok(()),
                            Ok(low::Outcome::Unanalyzable(reason)) => {
                                output.unanalyzable(
                                    policy.unanalyzable,
                                    finding(
                                        root,
                                        rule,
                                        contract,
                                        &contract.producer.file,
                                        reason,
                                        &allowed,
                                        sources,
                                    ),
                                );
                                Ok(())
                            }
                            Err(error) => Err(error),
                        }
                    } else {
                        check_contract(root, contract, &allowed, sources, &documents)
                    };
                    if let Err((file, reason)) = result {
                        output.findings.push(finding(
                            root, rule, contract, &file, reason, &allowed, sources,
                        ));
                    }
                    output
                })
                .reduce(AnalysisDiagnostics::default, |mut output, chunk| {
                    output.append(chunk);
                    output
                });
            Ok(output)
        })
        .collect();
    let mut output = AnalysisDiagnostics::default();
    for chunk in chunks? {
        output.append(chunk);
    }
    if !defer_suppression {
        super::super::suppress_rule_findings_with_sources(root, &mut output.findings, sources);
        super::super::suppress_rule_findings_with_sources(root, &mut output.advisories, sources);
    }
    super::super::sort_findings(&mut output.findings);
    super::super::sort_findings(&mut output.advisories);
    Ok(output)
}

fn finding(
    root: &Path,
    rule: &RuleDef,
    contract: &DeclaredPayloadContract,
    file: &str,
    reason: String,
    allowed: &BTreeSet<String>,
    sources: &SourceStore,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        line: references::finding_line(root, file, allowed, sources),
        file: file.to_owned(),
        message: match &rule.message {
            Some(message) => format!("{message}: contract `{}`: {reason}", contract.name),
            None => format!("contract `{}`: {reason}", contract.name),
        },
        import: None,
        target: Some(contract.name.clone()),
    }
}
