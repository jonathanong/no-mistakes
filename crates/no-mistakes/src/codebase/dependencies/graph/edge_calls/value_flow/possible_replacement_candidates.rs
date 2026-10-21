use super::*;

#[derive(Clone)]
pub(in super::super) enum ReplacementCandidates {
    Single(Option<ResolvedCallCandidate>),
    Possible(Vec<Option<ResolvedCallCandidate>>),
}
impl ReplacementCandidates {
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn has_known(&self) -> bool {
        match self {
            Self::Single(value) => value.is_some(),
            Self::Possible(values) => values.iter().any(Option::is_some),
        }
    }
}
impl IntoIterator for ReplacementCandidates {
    type Item = Option<ResolvedCallCandidate>;
    type IntoIter =
        std::iter::Chain<std::option::IntoIter<Self::Item>, std::vec::IntoIter<Self::Item>>;
    fn into_iter(self) -> Self::IntoIter {
        let (one, many) = match self {
            Self::Single(value) => (Some(value), Vec::new()),
            Self::Possible(values) => (None, values),
        };
        one.into_iter().chain(many)
    }
}

pub(in super::super) fn replacement_candidates(
    flow: &ValueFlowResolution<'_, '_>,
    value: FlowValue,
) -> ReplacementCandidates {
    let FlowValue::Possible(inner) = value else {
        // Existing Alternatives/Choice retain their historical single-leaf policy.
        return ReplacementCandidates::Single(replacement_candidate(flow, value));
    };
    let mut known = collect_possible(*inner, &mut |value| replacement_candidate(flow, value));
    known.push(None);
    ReplacementCandidates::Possible(known)
}

fn collect_possible(
    value: FlowValue,
    leaf: &mut impl FnMut(FlowValue) -> Option<ResolvedCallCandidate>,
) -> Vec<Option<ResolvedCallCandidate>> {
    match value {
        FlowValue::Possible(inner)
        | FlowValue::Provenance {
            executor: inner, ..
        }
        | FlowValue::TypedObject {
            executor: inner, ..
        }
        | FlowValue::Returned {
            executor: inner, ..
        }
        | FlowValue::Bound { target: inner, .. } => collect_possible(*inner, leaf),
        FlowValue::Choice(values) => values
            .into_iter()
            .flat_map(|value| collect_possible(value, leaf))
            .collect(),
        FlowValue::Target(ResolvedCallTarget::Alternatives(known)) => {
            let mut values = known.known().iter().cloned().map(Some).collect::<Vec<_>>();
            if known.has_unknown {
                values.push(None);
            }
            values
        }
        value => vec![leaf(value)],
    }
}

#[cfg(test)]
#[path = "possible_replacement_candidates_tests.rs"]
mod tests;
