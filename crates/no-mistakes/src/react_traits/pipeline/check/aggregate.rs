use crate::react_traits::pipeline::run_with_facts::PreparedComponentFacts;
use crate::react_traits::report::types::{ComponentFacts, ReactSuppressionTarget, Violation};

#[doc(hidden)]
pub struct PreparedReactFindings {
    pub findings: Vec<Violation>,
    pub suppression_targets: Vec<Vec<ReactSuppressionTarget>>,
}

/// Builds public React findings directly from the request-scoped facts. The
/// ordinary prepared-facts path does not need suppression locations, so it
/// must borrow these facts instead of cloning every component into sidecars.
pub(super) fn assert_no_fetch_violations(facts_list: &[ComponentFacts]) -> Vec<Violation> {
    facts_list.iter().filter_map(violation_for).collect()
}

pub(super) fn assert_no_fetch_violations_with_suppression(
    facts_list: &[PreparedComponentFacts],
) -> PreparedReactFindings {
    let mut violations = Vec::new();
    let mut suppression_targets = Vec::new();
    for prepared_facts in facts_list {
        let facts = &prepared_facts.facts;
        if let Some(violation) = violation_for(facts) {
            let mut finding_targets = facts
                .fetches
                .iter()
                .map(|fetch| ReactSuppressionTarget {
                    file: fetch.file.clone(),
                    line: fetch.line,
                })
                .collect::<Vec<_>>();
            finding_targets.extend(
                prepared_facts
                    .inherited_fetch_locations
                    .iter()
                    .cloned()
                    .map(|(file, line)| ReactSuppressionTarget { file, line }),
            );
            violations.push(violation);
            suppression_targets.push(finding_targets);
        }
    }
    PreparedReactFindings {
        findings: violations,
        suppression_targets,
    }
}

fn violation_for(facts: &ComponentFacts) -> Option<Violation> {
    let has_fetch = !facts.fetches.is_empty()
        || facts
            .inherited_from_children
            .as_ref()
            .is_some_and(|agg| agg.has_fetch);
    has_fetch.then(|| Violation {
        component: facts.name.clone(),
        file: facts.file.clone(),
        rule: "assert-no-fetch".to_string(),
        detail: facts.fetches.first().and_then(|f| f.shape.clone()),
    })
}
