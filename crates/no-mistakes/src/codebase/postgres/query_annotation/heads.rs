//! Original syntax proofs for annotation low effort; no helper value evaluation.
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;
mod bindings;
mod diagnostics;
mod syntax;
mod tag_constants;
#[cfg(test)]
mod tests;
pub(super) type Span = (u32, u32);
#[derive(Clone, Debug)]
pub(crate) enum HeadProof {
    Literal,
    Missing,
    Unproven,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct AnnotationHeads {
    pub(crate) calls: BTreeMap<Span, HeadProof>,
}
impl AnnotationHeads {
    pub(crate) fn accepts(&self, span: Span) -> bool {
        matches!(self.calls.get(&span), Some(HeadProof::Literal))
    }

    pub(crate) fn proves_missing(&self, span: Span) -> bool {
        matches!(self.calls.get(&span), Some(HeadProof::Missing))
    }
}
pub(crate) fn collect(
    program: &Program<'_>,
    ts: &TsFileFacts,
    executor_calls: impl Iterator<Item = Span>,
) -> AnnotationHeads {
    if ts.parse_error.is_some() || ts.operational_error.is_some() || ts.fatal_parse_error {
        return AnnotationHeads::default();
    }
    let syntax = syntax::collect(program);
    let bindings = bindings::Bindings::new(&syntax, ts, executor_calls);
    AnnotationHeads {
        calls: syntax
            .calls
            .iter()
            .map(|call| (call.span, bindings.proof(call.head)))
            .collect(),
    }
}
