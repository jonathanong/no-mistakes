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
        .chain(syntax.finite_templates.iter().map(|value| value.reference))
        .chain(
            syntax
                .finite_roots
                .iter()
                .flat_map(|value| [value.reference, value.tag]),
        )
        .chain(syntax.finite_calls.iter().flat_map(|call| {
            std::iter::once(call.callee).chain(match &call.first {
                super::super::finite_comments::Argument::Reference(span) => Some(*span),
                _ => None,
            })
        }))
        .chain(
            syntax
                .namespace_destructures
                .iter()
                .map(|value| value.namespace),
        )
        .chain(syntax.builder_direct_calls.values().copied())
        .chain(
            syntax
                .destructure_calls
                .values()
                .flat_map(|(text, values)| [*text, *values]),
        )
        .chain(
            syntax
                .record_destructures
                .values()
                .map(|source| source.callee),
        )
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
                    super::super::helper_records::Summary::PlainLocal { reference }
                    | super::super::helper_records::Summary::ReturnedLocal { reference } => {
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
