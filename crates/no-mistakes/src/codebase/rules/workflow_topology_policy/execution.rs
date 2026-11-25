use super::{concurrency_compile, evaluate, locations, malformed, secret_inventory, Options, RULE_ID};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::rules::{analysis_policy::AnalysisDiagnostics, file_diagnostics};
use crate::codebase::workflow_topology::model::DiagnosticCode;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

#[doc(hidden)]
pub fn diagnostics_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    session: Option<&AnalysisSession>,
    sources: &crate::codebase::ts_source::SourceStore,
    inputs: &crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs,
    config_path: Option<&Path>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    let findings = &mut output.findings;
    let unlocated = malformed::unlocated_merge_findings(inputs.topology());
    let unavailable: BTreeSet<_> = unlocated.iter().map(|finding| finding.file.as_str()).collect();
    let mut check_reusable = false;
    let mut locations = locations::Locations::default();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        check_reusable |= opts.forbid_reusable_workflow_concurrency;
        let first_finding = findings.len();
        let concurrency = concurrency_compile::compile(&opts.concurrency_policy)?;
        let inventory = opts
            .secret_inventory
            .as_ref()
            .map(|options| options.compile())
            .transpose()?;
        let topology_findings = evaluate::lint(inputs.topology(), &opts, &concurrency, &unavailable);
        findings.extend(locations.locate(root, sources, topology_findings));
        if let Some(inventory) = inventory {
            let authored = config
                .rules
                .iter()
                .position(|entry| std::ptr::eq(entry, rule))
                .and_then(|index| {
                    config.config_provenance.origin(
                        &crate::config::source_locations::ConfigPath::default()
                            .key("rules")
                            .index(index),
                    )
                });
            findings.extend(secret_inventory::lint(
                root,
                inputs,
                sources,
                &inventory,
                authored.map(|(path, _)| path).or(config_path),
            ));
        }
        if !unlocated.is_empty() {
            let mut uncertainty =
                file_diagnostics::route_application(config, rule, unlocated.clone(), |_| true)?;
            findings.append(&mut uncertainty.findings);
            output.advisories.append(&mut uncertainty.advisories);
        }
        if let Some(message) = &rule.message {
            for finding in &mut findings[first_finding..] {
                finding.message.clone_from(message);
            }
        }
    }
    if check_reusable {
        for diagnostic in &inputs.topology().diagnostics {
            if diagnostic.code == DiagnosticCode::MalformedWorkflow {
                malformed::report_malformed(diagnostic, session)?;
            }
        }
    }
    if !defer_suppression {
        for findings in [&mut output.findings, &mut output.advisories] {
            super::super::suppress_rule_findings_with_sources(root, findings, sources);
        }
    }
    Ok(file_diagnostics::sorted(output))
}
