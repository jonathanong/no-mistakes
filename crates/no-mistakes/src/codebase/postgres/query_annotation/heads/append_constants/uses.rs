//! Keep native opaque append receivers separate from ordinary and low-only reads.
use super::super::{syntax::Head, Span};
use super::Syntax;
use crate::fx::FxHashSet;

pub(super) struct Uses {
    pub ordinary: FxHashSet<Span>,
    pub prefix: FxHashSet<Span>,
    pub low: FxHashSet<Span>,
}

pub(super) fn collect(
    syntax: &Syntax,
    executors: &FxHashSet<Span>,
    effects: &super::super::effects::Prepared,
) -> Uses {
    let mut uses = Uses {
        ordinary: FxHashSet::default(),
        prefix: FxHashSet::default(),
        low: FxHashSet::default(),
    };
    for call in syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span))
    {
        match call.head {
            Head::FieldReference(reference) => {
                uses.low.insert(reference);
            }
            Head::AppendPrefixReference(reference) => {
                uses.prefix.insert(reference);
            }
            head => {
                if let Some(reference) = head.reference() {
                    uses.ordinary.insert(reference);
                }
            }
        }
    }
    uses.low.extend(syntax.return_sinks.iter().copied());
    uses.low.extend(effects.allowed.iter().copied());
    uses.low.extend(
        syntax
            .value_reads
            .iter()
            .filter_map(|(usage, call)| executors.contains(call).then_some(*usage)),
    );
    uses
}
