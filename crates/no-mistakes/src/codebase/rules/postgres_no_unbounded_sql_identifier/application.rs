use super::*;
use crate::codebase::rules::allow_audit::AllowAudit;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::schema::RuleDef;

pub(super) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    application: &RuleDef,
    graph: &DepGraph,
    facts: &CheckFactMap,
    sources: Option<&SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let opts: Options = application.try_rule_options()?;
    let opts = compile(RULE_ID, &opts)?;
    let filter = super::super::path_filter::RulePathFilter::new(root, config, application)?;
    let calls = sql_calls(root, &opts, facts);
    let returns = closed::configured::prepare(&opts, graph, facts);
    let mut findings = scan(root, &opts, graph, facts, &calls, &returns, &filter);
    if opts.identifier_sources.is_empty() {
        return Ok(findings);
    }
    let Some(sources) = sources else {
        return Ok(findings);
    };
    let complete = sources
        .inventory()
        .paths()
        .iter()
        .filter(|path| opts.includes(&relative_slash_path(root, path)) && filter.is_match(path))
        .all(|path| {
            facts.get_ts_facts(path).is_some_and(|ts| {
                ts.parse_error.is_none()
                    && ts.operational_error.is_none()
                    && ts.value_flow.incomplete_reason.is_none()
            })
        });
    if !complete {
        return Ok(findings);
    }
    let audit = AllowAudit::new(
        root,
        config,
        application,
        Some(sources),
        "identifierSources",
        opts.identifier_sources.len(),
    );
    audit.usage.suppress(
        opts.identifier_sources
            .iter()
            .enumerate()
            .map(|(index, _)| returns.source_used(index)),
    );
    findings.extend(audit.finish(
        |_| true,
        |index| {
            let source = &opts.identifier_sources[index];
            source.module.as_ref().map_or_else(
                || source.name.clone(),
                |module| format!("{module}#{}", source.name),
            )
        },
    ));
    Ok(findings)
}

#[cfg(test)]
#[path = "application_usage_tests.rs"]
mod tests;
