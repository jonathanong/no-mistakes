//! Exact inline callback forwarding through one prepared source helper hop.
mod caller;
mod producer;
mod uses;

use super::{syntax::Syntax, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
use oxc_ast::ast::Program;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Payload {
    Parameter(usize),
    Fixed,
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
    callbacks: Vec<Option<Span>>,
    payloads: Vec<bool>,
}

impl Pending {
    pub(crate) fn proven_executor(&self, summary: &Summary) -> Option<Span> {
        let span = self.callbacks.get(summary.callback).copied().flatten()?;
        match summary.payload {
            Payload::Fixed => Some(span),
            Payload::Parameter(index) if self.payloads.get(index) == Some(&true) => Some(span),
            _ => None,
        }
    }
}

pub(super) fn collect(
    program: &Program<'_>,
    module: Option<&TypeScriptModuleFacts>,
    syntax: &Syntax,
    executors: &FxHashSet<Span>,
) -> (BTreeMap<String, Summary>, Vec<Pending>) {
    let Some(module) = module.filter(|module| module.diagnostics.is_empty()) else {
        return Default::default();
    };
    let exported = producer::exported(program, module, syntax);
    let has_callback_receiver = syntax.calls.iter().any(|call| {
        executors.contains(&call.span) && matches!(call.head, super::syntax::Head::Reference(_))
    });
    let pending = if !has_callback_receiver {
        Vec::new()
    } else {
        caller::pending(program, module, executors)
    };
    (exported, pending)
}
