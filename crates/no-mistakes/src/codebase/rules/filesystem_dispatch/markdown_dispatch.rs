use super::{
    candidate_index::RuleCandidateIndex, markdown_child_links, markdown_link_display_text,
    markdown_mermaid_validation, markdown_reachability, markdown_structure_budget, rule_enabled,
    RuleFinding, MARKDOWN_CHILD_LINKS, MARKDOWN_LINK_DISPLAY_TEXT, MARKDOWN_MERMAID_VALIDATION,
    MARKDOWN_REACHABILITY, MARKDOWN_STRUCTURE_BUDGET,
};
use anyhow::Result;
use std::path::Path;
use std::sync::Mutex;

type RuleResults = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;

pub(super) fn prepare(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    candidates: &RuleCandidateIndex,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<super::super::markdown_facts::MarkdownFactMap> {
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    if rule_enabled(config, MARKDOWN_LINK_DISPLAY_TEXT) {
        plan.request_display_links(markdown_link_display_text::fact_candidate_files(
            root,
            config,
            candidates.candidates(MARKDOWN_LINK_DISPLAY_TEXT),
        ));
    }
    if rule_enabled(config, MARKDOWN_MERMAID_VALIDATION) {
        plan.request_pulldown(markdown_mermaid_validation::fact_candidate_files(
            root,
            config,
            candidates.candidates(MARKDOWN_MERMAID_VALIDATION),
        )?);
    }
    for rule_id in [
        MARKDOWN_CHILD_LINKS,
        MARKDOWN_REACHABILITY,
        MARKDOWN_STRUCTURE_BUDGET,
    ] {
        if rule_enabled(config, rule_id) {
            plan.request_pulldown(super::super::markdown_scope::markdown_files(
                candidates.candidates(rule_id),
            ));
        }
    }
    Ok(super::super::markdown_facts::MarkdownFactMap::prepare(
        &plan, sources,
    ))
}

pub(super) fn spawn<'scope>(
    scope: &rayon::Scope<'scope>,
    root: &'scope Path,
    config: &'scope crate::config::v2::NoMistakesConfig,
    candidates: &'scope RuleCandidateIndex,
    facts: &'scope super::super::markdown_facts::MarkdownFactMap,
    results: &'scope RuleResults,
) {
    spawn_rule(scope, config, MARKDOWN_CHILD_LINKS, results, || {
        markdown_child_links::check_with_files_sources_and_facts(
            root,
            config,
            candidates.candidates(MARKDOWN_CHILD_LINKS),
            facts,
        )
    });
    spawn_rule(scope, config, MARKDOWN_LINK_DISPLAY_TEXT, results, || {
        markdown_link_display_text::check_with_files_sources_and_facts(
            root,
            config,
            candidates.candidates(MARKDOWN_LINK_DISPLAY_TEXT),
            facts,
        )
    });
    spawn_rule(scope, config, MARKDOWN_MERMAID_VALIDATION, results, || {
        markdown_mermaid_validation::check_with_files_and_facts(
            root,
            config,
            candidates.candidates(MARKDOWN_MERMAID_VALIDATION),
            facts,
        )
    });
    spawn_rule(scope, config, MARKDOWN_REACHABILITY, results, || {
        markdown_reachability::check_with_files_sources_and_facts(
            root,
            config,
            candidates.candidates(MARKDOWN_REACHABILITY),
            facts,
        )
    });
    spawn_rule(scope, config, MARKDOWN_STRUCTURE_BUDGET, results, || {
        markdown_structure_budget::check_with_files_sources_and_facts(
            root,
            config,
            candidates.candidates(MARKDOWN_STRUCTURE_BUDGET),
            facts,
        )
    });
}

fn spawn_rule<'scope>(
    scope: &rayon::Scope<'scope>,
    config: &crate::config::v2::NoMistakesConfig,
    rule_id: &'static str,
    results: &'scope RuleResults,
    run: impl FnOnce() -> Result<Vec<RuleFinding>> + Send + 'scope,
) {
    if rule_enabled(config, rule_id) {
        scope.spawn(move |_| {
            let result = run();
            results
                .lock()
                .expect("mutex poisoned")
                .push((rule_id, result));
        });
    }
}
