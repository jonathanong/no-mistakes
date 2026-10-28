//! References required by the one prepared lexical binding index.
use super::{Span, Syntax};
use crate::fx::FxHashSet;

pub(super) fn references(syntax: &Syntax) -> FxHashSet<Span> {
    syntax
        .calls
        .iter()
        .map(|call| call.head)
        .chain(syntax.constants.values().copied())
        .filter_map(|head| head.reference())
        .chain(syntax.append_tags.values().copied())
        .chain(
            syntax
                .loop_candidates
                .iter()
                .flat_map(|candidate| std::iter::once(candidate.assignment).chain(candidate.tag)),
        )
        .chain(
            syntax
                .loop_candidates
                .iter()
                .flat_map(|candidate| candidate.condition_references.iter().copied()),
        )
        .chain(
            syntax
                .literal_helpers
                .iter()
                .flatten()
                .map(|call| call.reference),
        )
        .chain(syntax.helper_arguments.values().copied())
        .chain(
            syntax
                .namespace_destructures
                .iter()
                .map(|value| value.namespace),
        )
        .chain(syntax.builder_direct_calls.values().copied())
        .chain(
            syntax
                .record_initializers
                .values()
                .map(|(callee, _)| *callee),
        )
        .chain(syntax.helper_returns.values().filter_map(|value| value.tag))
        .chain(
            syntax
                .record_returns
                .values()
                .filter_map(|value| value.tag()),
        )
        .chain(
            syntax
                .record_returns
                .values()
                .filter_map(|value| match value {
                    super::super::helper_records::Summary::ReturnedLocal { reference } => {
                        Some(*reference)
                    }
                    _ => None,
                }),
        )
        .chain(syntax.return_sinks.iter().copied())
        .chain(
            syntax
                .finite_returns
                .values()
                .flat_map(|value| value.references()),
        )
        .chain(
            syntax
                .effect_calls
                .iter()
                .flat_map(|call| [call.callee, call.builder]),
        )
        .collect()
}
