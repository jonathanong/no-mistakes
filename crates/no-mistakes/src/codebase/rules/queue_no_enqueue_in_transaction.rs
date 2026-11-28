use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, ResolvedCallSite};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

mod analyze;
mod builtins;
mod callbacks;
mod options;
mod receivers;
mod reexports;
mod transactions;
use options::{Api, Options};

pub const RULE_ID: &str = "queue-no-enqueue-in-transaction";

pub(crate) fn validate_options(rule: &crate::config::v2::schema::RuleDef) -> Result<()> {
    rule.strict_rule_options::<options::Options>()?
        .targets()
        .map(drop)
}

pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    let root = normalize_path(root);
    for application in config.rule_applications(RULE_ID) {
        let options = application.try_rule_options::<Options>()?;
        let targets = options.targets()?;
        let policy = config.effective_rule_analysis(application)?.unanalyzable;
        let files = super::selected_rule_files(
            &root,
            config,
            application,
            facts.graph_file_universe(),
        )?;
        let ctx = analyze::Context {
            root: &root,
            application,
            policy,
            targets: &targets,
            graph,
            facts,
        };
        let per_file: Vec<AnalysisDiagnostics> = files
            .par_iter()
            .map(|file| {
                let mut local = AnalysisDiagnostics::default();
                let file = normalize_path(file);
                analyze::check_file(&ctx, &file, &mut local);
                local
            })
            .collect();
        for local in per_file {
            diagnostics.append(local);
        }
    }
    diagnostics.findings.sort();
    diagnostics.advisories.sort();
    Ok(diagnostics)
}

fn finding(ctx: &analyze::Context<'_>, site: &ResolvedCallSite, api: Api<'_>) -> RuleFinding {
    let body = "Move this enqueue after the transaction commits. Return the job data from the callback and enqueue it after the transaction call resolves.";
    build_finding(ctx, site, body, format!("{}#{}", api.0, api.1))
}

fn unanalyzable(ctx: &analyze::Context<'_>, site: &ResolvedCallSite, reason: &str) -> RuleFinding {
    build_finding(ctx, site, reason, "unanalyzable".into())
}

fn build_finding(
    ctx: &analyze::Context<'_>,
    site: &ResolvedCallSite,
    body: &str,
    target: String,
) -> RuleFinding {
    let message = ctx
        .application
        .message
        .as_deref()
        .map_or_else(|| body.to_owned(), |p| format!("{p}: {body}"));
    RuleFinding {
        rule: RULE_ID.into(),
        file: relative_slash_path(ctx.root, &site.file),
        line: site.line as usize,
        message,
        import: Some(site.source_callee.clone()),
        target: Some(target),
        causes: Vec::new(),
    }
}
