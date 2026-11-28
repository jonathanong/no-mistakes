use super::{
    concurrency_compile, evaluate, evaluate_call_bindings, locations, malformed, secret_inventory,
    Options, RULE_ID,
};
use crate::codebase::analysis_session::AnalysisSession;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::codebase::rules::{file_diagnostics};
use crate::codebase::workflow_topology::model::DiagnosticCode;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use std::path::Path;

fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<Unanalyzable> {
    let explicit = rule.options.get("unanalyzable").is_some() || config.unanalyzable.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}

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
    let unlocated = malformed::unlocated_merge_findings(inputs.topology());
    let availability = super::merge_availability::MergeAvailability::new(inputs.topology());
    let mut check_reusable = false;
    let mut locations = locations::Locations::default();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        check_reusable |= opts.forbid_reusable_workflow_concurrency;
        let first_finding = output.findings.len();
        let concurrency = concurrency_compile::compile(&opts.concurrency_policy)?;
        let inventory = opts
            .secret_inventory
            .as_ref()
            .map(|options| options.compile())
            .transpose()?;
        let topology_findings =
            evaluate::lint(inputs.topology(), &opts, &concurrency, &availability);
        output
            .findings
            .extend(locations.locate(root, sources, topology_findings));
        let call_bindings = evaluate_call_bindings::lint(inputs, &opts);
        output
            .findings
            .extend(locations.locate(root, sources, call_bindings.findings));
        for finding in locations.locate(root, sources, call_bindings.unanalyzable) {
            output.unanalyzable(unanalyzable, finding);
        }
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
            output.findings.extend(secret_inventory::lint(
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
            output.findings.append(&mut uncertainty.findings);
            output.advisories.append(&mut uncertainty.advisories);
        }
        if let Some(message) = &rule.message {
            for finding in &mut output.findings[first_finding..] {
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
