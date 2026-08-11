use crate::react_traits::pipeline::run_with_facts::PreparedComponentFacts;
use crate::react_traits::report::types::{ReactSuppressionTarget, Violation};

#[doc(hidden)]
pub struct PreparedReactFindings {
    pub findings: Vec<Violation>,
    pub suppression_targets: Vec<Vec<ReactSuppressionTarget>>,
}

pub(super) fn assert_no_fetch_violations(
    facts_list: &[crate::react_traits::ComponentFacts],
) -> Vec<Violation> {
    let prepared = facts_list
        .iter()
        .cloned()
        .map(|facts| PreparedComponentFacts {
            facts,
            inherited_fetch_locations: Vec::new(),
        })
        .collect::<Vec<_>>();
    assert_no_fetch_violations_with_suppression(&prepared).findings
}

pub(super) fn assert_no_fetch_violations_with_suppression(
    facts_list: &[PreparedComponentFacts],
) -> PreparedReactFindings {
    let mut violations = Vec::new();
    let mut suppression_targets = Vec::new();
    for prepared_facts in facts_list {
        let facts = &prepared_facts.facts;
        let has_fetch = !facts.fetches.is_empty()
            || facts
                .inherited_from_children
                .as_ref()
                .is_some_and(|agg| agg.has_fetch);
        if has_fetch {
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
            violations.push(Violation {
                component: facts.name.clone(),
                file: facts.file.clone(),
                rule: "assert-no-fetch".to_string(),
                detail: facts.fetches.first().and_then(|f| f.shape.clone()),
            });
            suppression_targets.push(finding_targets);
        }
    }
    PreparedReactFindings {
        findings: violations,
        suppression_targets,
    }
}
