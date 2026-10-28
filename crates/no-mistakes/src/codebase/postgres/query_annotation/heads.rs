//! Original syntax proofs for annotation low effort; no helper value evaluation.
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
mod append;
mod append_constants;
mod assignments;
mod bindings;
mod callback_heads;
mod captured_prefix;
mod conditional;
mod diagnostics;
mod effects;
mod finite_returns;
mod helper_heads;
mod helper_records;
mod import_projection;
mod literal_helpers;
mod lookup;
mod loop_carried;
mod loop_word;
pub(crate) use loop_word::{prepare_loop_heads, PreparedLoopHeads};
mod syntax;
mod tag_constants;
#[cfg(test)]
mod tests;
mod text;
mod trusted_append;
#[cfg(test)]
mod trusted_append_tests;
pub(super) type Span = (u32, u32);
/// Provisional executor calls supply helper heads but never safe builder uses.
#[derive(Default)]
pub(crate) struct HeadCallSpans {
    pub(crate) confirmed: crate::fx::FxHashSet<Span>,
    pub(crate) pending: crate::fx::FxHashSet<Span>,
}

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
    pub(crate) exported_values: BTreeMap<String, HeadProof>,
    pub(crate) exported_records: BTreeMap<String, helper_records::Summary>,
    pub(crate) pending_imports: Vec<helper_heads::Pending>,
    pub(crate) exported_effects: BTreeSet<String>,
    pub(crate) pending_effects: Vec<effects::Pending>,
    pub(crate) exported_callbacks: BTreeMap<String, callback_heads::Summary>,
    pub(crate) pending_callbacks: Vec<callback_heads::Pending>,
    pub(crate) conditional_effects: Vec<effects::Conditional>,
    pub(crate) loop_carried_missing: BTreeMap<Span, Vec<Option<String>>>,
    helper_calls: Option<BTreeMap<Span, (String, HeadProof)>>,
    pub(crate) trusted_append_calls: crate::fx::FxHashSet<Span>,
}
pub(crate) fn collect_prepared(
    program: &Program<'_>,
    ts: &TsFileFacts,
    executor_calls: HeadCallSpans,
    candidates: &crate::fx::FxHashSet<Span>,
    include_all_calls: bool,
    helpers: bool,
    prepared: Option<PreparedLoopHeads>,
) -> AnnotationHeads {
    if ts.parse_error.is_some() || ts.operational_error.is_some() || ts.fatal_parse_error {
        return AnnotationHeads::default();
    }
    let mut syntax = prepared.map(|prepared| prepared.0).unwrap_or_else(|| {
        syntax::collect_with_helpers(
            program,
            !candidates.is_empty() || include_all_calls,
            helpers,
            include_all_calls,
        )
    });
    syntax.helper_arguments.retain(|call, _| {
        executor_calls.confirmed.contains(call) || executor_calls.pending.contains(call)
    });
    let executor_calls = executor_calls.confirmed;
    if include_all_calls {
        syntax.confirmed_builder_reads = helper_records::confirmed_reads(&syntax, &executor_calls);
    }
    let bindings = bindings::Bindings::new(
        &syntax,
        ts,
        executor_calls.iter().copied(),
        include_all_calls,
    );
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
    let low_candidates = include_all_calls.then_some(&executor_calls);
    let (trusted_append_calls, trusted_prefix_calls) =
        trusted_append::proofs(&syntax, &bindings, &calls, candidates, low_candidates);
    if include_all_calls {
        for (span, proof) in trusted_prefix_calls {
            calls.insert(span, proof);
        }
        // Source-proven helper results are low-only endpoints. Legacy max
        // append fallback above must see the original call-head proofs.
        for call in syntax
            .calls
            .iter()
            .filter(|call| executor_calls.contains(&call.span))
        {
            if let Some(summary) = helper_records::local_proof(&syntax, &bindings, call)
                .or_else(|| helper_records::builder_local_proof(&syntax, &bindings, call))
            {
                calls.insert(call.span, summary.proof());
            }
        }
    }
    let conditional_effects = if include_all_calls {
        syntax
            .calls
            .iter()
            .filter(|call| executor_calls.contains(&call.span))
            .filter(|call| !matches!(calls.get(&call.span), Some(HeadProof::Literal)))
            .filter_map(|call| {
                bindings
                    .conditional_prefix(call.head, call.span)
                    .map(|required| effects::Conditional {
                        span: call.span,
                        required,
                    })
            })
            .collect()
    } else {
        Vec::new()
    };
    let helper_calls = literal_helpers::proofs(&syntax, &bindings, ts);
    let (exported_callbacks, pending_callbacks) = if include_all_calls {
        callback_heads::collect(
            program,
            ts.module_bindings.as_deref(),
            &syntax,
            &executor_calls,
        )
    } else {
        Default::default()
    };
    let (exported_helpers, exported_values, exported_records, pending_imports) =
        if include_all_calls {
            helper_heads::module_projection(
                &syntax,
                &bindings,
                ts.module_bindings.as_deref(),
                &executor_calls,
            )
        } else {
            Default::default()
        };
    AnnotationHeads {
        exported_helpers,
        exported_values,
        exported_records,
        exported_callbacks,
        pending_callbacks,
        pending_imports,
        exported_effects: if include_all_calls {
            bindings.effects().exported.clone()
        } else {
            Default::default()
        },
        pending_effects: if include_all_calls {
            bindings.effects().pending.clone()
        } else {
            Default::default()
        },
        conditional_effects,
        helper_calls,
        loop_carried_missing,
        calls,
        trusted_append_calls,
    }
}

pub(crate) use import_projection::project;

#[cfg(test)]
mod literal_helper_tests;
