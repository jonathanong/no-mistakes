//! Low annotation head facts from the original prepared program.
use super::{
    bindings, callback_heads, effects, finite_comments, helper_heads, helper_records,
    literal_helpers, loop_carried, namespace_heads, returned_params, syntax, trusted_append,
    AnnotationHeads, HeadCallSpans, HeadProof, PreparedLoopHeads, Span,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;

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
        for (span, proof) in super::branch_assignments::proofs(&syntax, &bindings, &executor_calls)
        {
            if matches!(calls.get(&span), Some(HeadProof::Unproven)) {
                calls.insert(span, proof);
            }
        }
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
            if let Some(summary) = helper_records::local_result_proof(&syntax, &bindings, call) {
                calls.insert(call.span, summary.proof());
            }
        }
        for (span, proof) in finite_comments::proofs(
            &syntax,
            &bindings,
            ts.module_bindings.as_deref(),
            &executor_calls,
        ) {
            if matches!(calls.get(&span), Some(HeadProof::Unproven)) {
                calls.insert(span, proof);
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
    let (exported_callbacks, pending_callbacks) = callback_heads::collect(
        include_all_calls,
        program,
        ts.module_bindings.as_deref(),
        &syntax,
        &bindings,
        &executor_calls,
        &mut calls,
    );
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
    let (exported_returned_params, pending_returned_params) = if include_all_calls {
        returned_params::module_projection(
            &syntax,
            &bindings,
            ts.module_bindings.as_deref(),
            &executor_calls,
        )
    } else {
        Default::default()
    };
    let pending_namespaces = if include_all_calls {
        namespace_heads::pending(
            &syntax,
            &bindings,
            ts.module_bindings.as_deref(),
            &executor_calls,
        )
    } else {
        Vec::new()
    };
    let (exported_async_returns, pending_async_returns) = if include_all_calls {
        super::async_returns::collect(
            &syntax,
            &bindings,
            ts.module_bindings.as_deref(),
            &executor_calls,
        )
    } else {
        Default::default()
    };
    AnnotationHeads {
        exported_async_returns,
        pending_async_returns,
        exported_helpers,
        exported_values,
        exported_records,
        exported_returned_params,
        pending_returned_params,
        exported_callbacks,
        pending_callbacks,
        pending_imports,
        pending_namespaces,
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
