//! Original syntax proofs for annotation low effort; no helper value evaluation.
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;
mod append;
mod append_constants;
mod assignments;
mod bindings;
mod diagnostics;
mod helper_heads;
mod import_projection;
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
    pub(crate) exported_helpers: BTreeMap<String, HeadProof>,
    pub(crate) pending_helpers: Vec<helper_heads::Pending>,
    pub(crate) loop_carried_missing: BTreeMap<Span, Vec<Option<String>>>,
    helper_calls: Option<BTreeMap<Span, (String, HeadProof)>>,
    pub(crate) trusted_append_calls: crate::fx::FxHashSet<Span>,
}
impl AnnotationHeads {
    pub(crate) fn has_pending_imports(&self) -> bool {
        !self.pending_helpers.is_empty()
    }
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
    if !has_loop_word(source) {
        return None;
    }
    let syntax = syntax::collect_with_helpers(program, append, helpers, false);
    (!syntax.loop_candidates.is_empty() || append || helpers).then_some(PreparedLoopHeads(syntax))
}

fn has_loop_word(source: &str) -> bool {
    fn identifier(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
    }
    let bytes = source.as_bytes();
    ["for", "while"].into_iter().any(|word| {
        source.match_indices(word).any(|(start, _)| {
            (start == 0 || !identifier(bytes[start - 1]))
                && (start + word.len() == bytes.len() || !identifier(bytes[start + word.len()]))
        })
    })
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
    let mut syntax = prepared.map(|prepared| prepared.0).unwrap_or_else(|| {
        syntax::collect_with_helpers(program, !candidates.is_empty(), helpers, include_all_calls)
    });
    let executor_calls = executor_calls.collect::<crate::fx::FxHashSet<_>>();
    syntax
        .helper_arguments
        .retain(|call, _| executor_calls.contains(call));
    let bindings = bindings::Bindings::new(&syntax, ts, executor_calls.iter().copied());
    let loop_carried_missing = loop_carried::proofs(&syntax, &bindings);
    let mut calls: BTreeMap<_, _> = syntax
        .calls
        .iter()
        .filter(|call| include_all_calls || candidates.contains(&call.span))
        .map(|call| {
            (
                call.span,
                include_all_calls
                    .then(|| helper_heads::proof(&syntax, &bindings, call.span))
                    .flatten()
                    .unwrap_or_else(|| bindings.proof(call.head, call.span)),
            )
        })
        .collect();
    // Only low publishes the opaque-prefix proof; max keeps its legacy fallback.
    let (trusted_append_calls, trusted_prefix_calls) =
        trusted_append::proofs(&syntax, &bindings, &calls, candidates);
    if include_all_calls {
        for (span, proof) in trusted_prefix_calls {
            calls.insert(span, proof);
        }
    }
    let helper_calls = literal_helpers::proofs(&syntax, &bindings, ts);
    let (exported_helpers, pending_helpers) = if include_all_calls {
        helper_heads::module_projection(&syntax, &bindings, ts.module_bindings.as_deref())
    } else {
        Default::default()
    };
    AnnotationHeads {
        exported_helpers,
        pending_helpers,
        helper_calls,
        loop_carried_missing,
        calls,
        trusted_append_calls,
    }
}

pub(crate) use import_projection::project;

#[cfg(test)]
mod literal_helper_tests;
