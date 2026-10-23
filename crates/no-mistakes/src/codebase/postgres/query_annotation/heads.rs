//! Original syntax proofs for annotation low effort; no helper value evaluation.
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;
mod append;
mod append_constants;
mod assignments;
mod bindings;
mod diagnostics;
mod syntax;
mod tag_constants;
#[cfg(test)]
mod tests;
mod text;
mod trusted_append;
#[cfg(test)]
mod trusted_append_tests;
pub(super) type Span = (u32, u32);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadProof {
    Literal,
    Missing,
    Unproven,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct AnnotationHeads {
    pub(crate) calls: BTreeMap<Span, HeadProof>,
    pub(crate) trusted_append_calls: crate::fx::FxHashSet<Span>,
}
impl AnnotationHeads {
    pub(crate) fn accepts(&self, span: Span) -> bool {
        matches!(self.calls.get(&span), Some(HeadProof::Literal))
    }
    pub(crate) fn proves_missing(&self, span: Span) -> bool {
        matches!(self.calls.get(&span), Some(HeadProof::Missing))
    }
    pub(crate) fn accepts_trusted_append(&self, span: Span) -> bool {
        self.trusted_append_calls.contains(&span)
    }
}
pub(crate) fn collect(
    program: &Program<'_>,
    ts: &TsFileFacts,
    executor_calls: impl Iterator<Item = Span>,
) -> AnnotationHeads {
    collect_with_trusted_appends(program, ts, executor_calls, &Default::default(), true)
}

pub(crate) fn collect_with_trusted_appends(
    program: &Program<'_>,
    ts: &TsFileFacts,
    executor_calls: impl Iterator<Item = Span>,
    candidates: &crate::fx::FxHashSet<Span>,
    include_all_calls: bool,
) -> AnnotationHeads {
    if ts.parse_error.is_some() || ts.operational_error.is_some() || ts.fatal_parse_error {
        return AnnotationHeads::default();
    }
    let syntax = syntax::collect(program, !candidates.is_empty());
    let bindings = bindings::Bindings::new(&syntax, ts, executor_calls);
    let calls = syntax
        .calls
        .iter()
        .filter(|call| include_all_calls || candidates.contains(&call.span))
        .map(|call| (call.span, bindings.proof(call.head, call.span)))
        .collect();
    let trusted_append_calls = trusted_append::proofs(&syntax, &bindings, &calls, ts, candidates);
    AnnotationHeads {
        calls,
        trusted_append_calls,
    }
}
