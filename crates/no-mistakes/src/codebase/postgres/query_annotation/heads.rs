//! Original syntax proofs for annotation low effort; no helper value evaluation.
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;
mod append;
mod append_constants;
mod assignments;
mod bindings;
mod diagnostics;
mod literal_helpers;
mod loop_carried;
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
    pub(crate) loop_carried_missing: BTreeMap<Span, Option<String>>,
    helper_calls: Option<BTreeMap<Span, (String, HeadProof)>>,
    pub(crate) trusted_append_calls: crate::fx::FxHashSet<Span>,
}
impl AnnotationHeads {
    pub(crate) fn helper(&self, span: Span, helpers: &[String]) -> Option<HeadProof> {
        if helpers.is_empty() {
            return None;
        }
        let (identity, proof) = self.helper_calls.as_ref()?.get(&span)?;
        helpers.contains(identity).then_some(*proof)
    }
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
pub(crate) struct PreparedLoopHeads(syntax::Syntax);

/// Negative source filtering skips the visitor entirely for ordinary max files.
/// Positive matches are confirmed by the existing heads visitor, then reused.
pub(crate) fn prepare_loop_heads(
    program: &Program<'_>,
    source: &str,
    append: bool,
    helpers: bool,
) -> Option<PreparedLoopHeads> {
    if !source.contains("for") {
        return None;
    }
    let syntax = syntax::collect_with_helpers(program, append, helpers);
    (!syntax.loop_candidates.is_empty() || append || helpers).then_some(PreparedLoopHeads(syntax))
}

pub(crate) fn collect_prepared(
    program: &Program<'_>,
    ts: &TsFileFacts,
    executor_calls: impl Iterator<Item = Span>,
    candidates: &crate::fx::FxHashSet<Span>,
    include_all_calls: bool,
    helpers: bool,
    prepared: Option<PreparedLoopHeads>,
) -> AnnotationHeads {
    if ts.parse_error.is_some() || ts.operational_error.is_some() || ts.fatal_parse_error {
        return AnnotationHeads::default();
    }
    let syntax = prepared
        .map(|prepared| prepared.0)
        .unwrap_or_else(|| syntax::collect_with_helpers(program, !candidates.is_empty(), helpers));
    let bindings = bindings::Bindings::new(&syntax, ts, executor_calls);
    let loop_carried_missing = loop_carried::proofs(&syntax, &bindings);
    let calls = syntax
        .calls
        .iter()
        .filter(|call| include_all_calls || candidates.contains(&call.span))
        .map(|call| (call.span, bindings.proof(call.head, call.span)))
        .collect();
    let trusted_append_calls = trusted_append::proofs(&syntax, &bindings, &calls, ts, candidates);
    let helper_calls = literal_helpers::proofs(&syntax, &bindings, ts);
    AnnotationHeads {
        helper_calls,
        loop_carried_missing,
        calls,
        trusted_append_calls,
    }
}

#[cfg(test)]
mod literal_helper_tests;
