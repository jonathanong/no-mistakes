//! Exact inline callback forwarding through one prepared source helper hop.
mod caller;
mod producer;
mod uses;
#[cfg(test)]
#[path = "callback_heads/uses_tests.rs"]
mod uses_tests;

use super::{syntax::Syntax, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Payload {
    Parameter(usize),
    Fixed,
    FixedBuilder,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Summary {
    pub callback: usize,
    pub payload: Payload,
}

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub specifier: String,
    pub imported: String,
    callbacks: Vec<Option<(Span, bool)>>,
    payloads: Vec<bool>,
}

impl Pending {
    pub(crate) fn proven_executor(&self, summary: &Summary) -> Option<Span> {
        projected(&self.callbacks, &self.payloads, summary)
    }
}

fn projected(
    callbacks: &[Option<(Span, bool)>],
    payloads: &[bool],
    summary: &Summary,
) -> Option<Span> {
    let (span, needs_fresh) = callbacks.get(summary.callback).copied().flatten()?;
    if needs_fresh && summary.payload != Payload::FixedBuilder {
        return None;
    }
    match summary.payload {
        Payload::Fixed | Payload::FixedBuilder => Some(span),
        Payload::Parameter(index) if payloads.get(index) == Some(&true) => Some(span),
        _ => None,
    }
}

pub(super) fn collect(
    enabled: bool,
    program: &Program<'_>,
    module: Option<&TypeScriptModuleFacts>,
    syntax: &Syntax,
    bindings: &super::bindings::Bindings<'_>,
    executors: &FxHashSet<Span>,
    calls: &mut BTreeMap<Span, super::HeadProof>,
) -> (BTreeMap<String, Summary>, Vec<Pending>) {
    if !enabled {
        return Default::default();
    }
    let Some(module) = module.filter(|module| module.diagnostics.is_empty()) else {
        return Default::default();
    };
    let (exported, local) = producer::summaries(program, module, syntax, bindings);
    let has_callback_receiver = syntax.calls.iter().any(|call| {
        executors.contains(&call.span) && matches!(call.head, super::syntax::Head::Reference(_))
    });
    let (pending, completed) = if !has_callback_receiver {
        Default::default()
    } else {
        caller::project(program, module, syntax, bindings, executors, &local)
    };
    calls.extend(
        completed
            .into_iter()
            .map(|span| (span, super::HeadProof::Literal)),
    );
    (exported, pending)
}
