use anyhow::Result;
use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::codebase::check_outcome::{check_application_key, CheckFailure, Prerequisite};
use no_mistakes::config::v2::NoMistakesConfig;

pub(crate) fn record(
    session: &AnalysisSession,
    config: &NoMistakesConfig,
    error: &anyhow::Error,
) -> Result<()> {
    if !session.partial_checks_enabled() {
        anyhow::bail!("{error:#}");
    }
    for (rule, ordinal) in applications(config) {
        let mut selected = config.clone();
        selected.rules = vec![rule.clone()];
        let demand = no_mistakes::codebase::rules::try_canonical_graph_plan(&selected)?;
        if demand.is_some_and(|plan| plan.playwright_routes || plan.playwright_selectors) {
            mark(
                session,
                rule,
                ordinal,
                CheckFailure {
                    consumer: rule.rule.clone(),
                    file: None,
                    property: None,
                    message: format!("{error:#}"),
                },
            );
        }
    }
    Ok(())
}

pub(crate) fn record_prepared(
    root: &std::path::Path,
    session: &AnalysisSession,
    config: &NoMistakesConfig,
    files: &[std::path::PathBuf],
    error: &no_mistakes::codebase::dependencies::graph::PlaywrightPreparationFailure,
) -> Result<()> {
    for (rule, ordinal) in applications(config) {
        let mut selected = config.clone();
        selected.rules = vec![rule.clone()];
        let demand = no_mistakes::codebase::rules::try_canonical_graph_plan(&selected)?;
        if demand.is_some_and(|plan| plan.playwright_routes || plan.playwright_selectors)
            && error.affects_rule(root, config, rule, files)?
        {
            mark(
                session,
                rule,
                ordinal,
                CheckFailure {
                    consumer: rule.rule.clone(),
                    file: error.file.clone(),
                    property: error.message.contains("testDir").then(|| "testDir".into()),
                    message: error.message.clone(),
                },
            );
        }
    }
    Ok(())
}

fn applications(
    config: &NoMistakesConfig,
) -> Vec<(&no_mistakes::config::v2::schema::RuleDef, usize)> {
    let consumers: std::collections::BTreeSet<_> =
        config.rules.iter().map(|rule| rule.rule.as_str()).collect();
    consumers
        .into_iter()
        .flat_map(|consumer| {
            config
                .rule_applications(consumer)
                .into_iter()
                .enumerate()
                .map(|(index, rule)| (rule, index + 1))
        })
        .collect()
}

pub(crate) fn initialize(session: &AnalysisSession, config: &NoMistakesConfig) {
    if !session.partial_checks_enabled() {
        return;
    }
    for (rule, ordinal) in applications(config) {
        let key = check_application_key(rule, ordinal);
        if session.check_prerequisite(&key).is_none() {
            session.set_check_prerequisite(&key, Prerequisite::Ready(()));
        }
    }
}

fn mark(
    session: &AnalysisSession,
    rule: &no_mistakes::config::v2::schema::RuleDef,
    ordinal: usize,
    failure: CheckFailure,
) {
    let key = check_application_key(rule, ordinal);
    let mut failures = match session.check_prerequisite(&key) {
        Some(Prerequisite::Failed(failures)) => failures,
        _ => Vec::new(),
    };
    failures.push(failure);
    failures.sort();
    failures.dedup();
    session.set_check_prerequisite(&key, Prerequisite::Failed(failures));
}

#[cfg(test)]
mod tests;
