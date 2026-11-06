use super::*;

use crate::codebase::rules::test_no_live_repository_scan::Resolution;

mod coverage;
mod part0;
mod part1;
mod part2;
mod request;

pub(super) use request::GraphRuleRequest;

pub(super) fn graph_rule_findings_with_session(
    request: GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<crate::codebase::rules::analysis_policy::AnalysisDiagnostics> {
    let mut findings = Vec::new();
    let mut advisories = Vec::new();
    if session.is_some_and(|session| session.partial_checks_enabled()) {
        let production = "production-exports-only-used-by-tests";
        if request.config.rule_configured(production) {
            let config = crate::config::v2::NoMistakesConfig {
                rules: request
                    .config
                    .rule_applications(production)
                    .into_iter()
                    .cloned()
                    .collect(),
                ..request.config.clone()
            };
            collect_application(
                &GraphRuleRequest {
                    config: &config,
                    ..request
                },
                session,
                &mut findings,
                &mut advisories,
            )?;
        }
        for consumer in super::outcomes::GRAPH_CONSUMERS {
            if *consumer == production {
                continue;
            }
            for (index, application) in request
                .config
                .rule_applications(consumer)
                .into_iter()
                .enumerate()
            {
                let key =
                    crate::codebase::check_outcome::check_application_key(application, index + 1);
                if matches!(
                    session.and_then(|session| session.check_prerequisite(&key)),
                    Some(crate::codebase::check_outcome::Prerequisite::Failed(_))
                ) {
                    continue;
                }
                let mut config = request.config.clone();
                for (rule, original) in config.rules.iter_mut().zip(&request.config.rules) {
                    rule.enabled &= std::ptr::eq(original, application);
                    if rule.enabled {
                        rule.resolved_application_index.get_or_insert(index + 1);
                    }
                }
                let scoped = GraphRuleRequest {
                    config: &config,
                    ..request
                };
                collect_application(&scoped, session, &mut findings, &mut advisories)?;
            }
        }
    } else {
        collect_application(&request, session, &mut findings, &mut advisories)?;
    }
    Ok(
        crate::codebase::rules::analysis_policy::AnalysisDiagnostics {
            findings,
            advisories,
            suppressed: Vec::new(),
        },
    )
}

fn collect_application(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    part0::collect(request, session, findings, advisories)?;
    part1::collect(request, session, findings, advisories)?;
    part2::collect(request, session, findings, advisories)?;
    coverage::collect(request, session, findings, advisories)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod prerequisite_tests;
