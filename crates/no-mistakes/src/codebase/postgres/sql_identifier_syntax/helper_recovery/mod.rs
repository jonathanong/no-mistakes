//! Bounded low-only helper facts, collected in the existing parsed-program walk.
mod application;
mod budget;
pub(crate) use application::ApplicationKey;
mod catalog;
mod project;
mod transports;
pub(crate) use project::project;
mod recovered;
mod sql_text;
mod state;
mod value;
mod values;
pub(crate) use recovered::{ExecutionSite, Recovered};
pub(crate) mod lower;
use super::{span, template_binds::TemplateBinds, Span};
use crate::codebase::dependencies::extract::CallableId;
use crate::fx::FxHashMap;
use oxc_ast::ast::{Argument, CallExpression, Function};

#[derive(Clone)]
pub(crate) struct FunctionModel {
    pub id: CallableId,
    pub binding: Option<usize>,
    pub lowered: lower::Lowered,
}
#[derive(Clone)]
pub(crate) struct CallModel {
    pub span: Span,
    pub callee: Option<usize>,
    pub members: Vec<String>,
    pub arguments: Vec<lower::Lowered>,
    pub caller: Option<CallableId>,
}
#[derive(Default)]
pub(crate) struct FileModels {
    pub stack: Vec<CallableId>,
    pub binding_functions: FxHashMap<usize, Option<CallableId>>,
    pub argument_owners: FxHashMap<Span, usize>,
    pub functions: FxHashMap<CallableId, FunctionModel>,
    pub calls: Vec<CallModel>,
    pub executors:
        std::collections::BTreeMap<super::EmbeddedSqlOptions, crate::fx::FxHashSet<usize>>,
    pub sinks: std::collections::BTreeMap<super::EmbeddedSqlOptions, crate::fx::FxHashSet<Span>>,
    pub writes: crate::fx::FxHashSet<usize>,
}
impl FileModels {
    pub fn function(&mut self, function: &Function<'_>, ctx: &lower::Context<'_>) {
        let Some(lowered) = lower::function(function, ctx) else {
            return;
        };
        let id = CallableId(function.span.start);
        self.functions.insert(
            id,
            FunctionModel {
                id,
                binding: function.id.as_ref().and_then(|id| ctx.bindings.id(id.span)),
                lowered,
            },
        );
    }
    pub fn call(&mut self, call: &CallExpression<'_>, ctx: &lower::Context<'_>) {
        if call.optional
            || call
                .arguments
                .iter()
                .any(|arg| matches!(arg, Argument::SpreadElement(_)))
        {
            return;
        }
        let chain = super::chain(&call.callee);
        let callee_span = chain.as_ref().map(|(root, _)| *root);
        let callee = callee_span.and_then(|at| ctx.bindings.id(oxc_span::Span::new(at.0, at.1)));
        let members = chain.map_or(Vec::new(), |(_, parts)| parts.into_iter().skip(1).collect());
        self.calls.push(CallModel {
            span: span(call.span),
            callee,
            members,
            caller: self.stack.last().copied(),
            arguments: call
                .arguments
                .iter()
                .filter_map(Argument::as_expression)
                .map(|arg| lower::expression(arg, ctx))
                .collect(),
        });
    }
    pub(in crate::codebase::postgres::sql_identifier_syntax) fn validate(
        &mut self,
        tags: &TemplateBinds,
    ) {
        self.functions
            .retain(|_, function| function.lowered.tags.iter().all(|id| tags.is_valid(*id)));
        self.calls.retain(|call| {
            call.arguments
                .iter()
                .all(|argument| argument.tags.iter().all(|id| tags.is_valid(*id)))
        });
        self.calls.sort_by_key(|call| call.span);
        for function in self.functions.values() {
            if let Some(binding) = function.binding {
                self.binding_functions
                    .entry(binding)
                    .and_modify(|id| *id = None)
                    .or_insert(Some(function.id));
            }
        }
        for (index, call) in self.calls.iter().enumerate() {
            for argument in &call.arguments {
                for (_, at) in &argument.references {
                    self.argument_owners
                        .entry(*at)
                        .and_modify(|owner| {
                            let previous = &self.calls[*owner];
                            if call.span.1 - call.span.0 < previous.span.1 - previous.span.0 {
                                *owner = index;
                            }
                        })
                        .or_insert(index);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod replay_tests;

#[cfg(test)]
mod precision_tests;

#[cfg(test)]
mod coverage_tests;

#[cfg(test)]
mod patch_budget_tests;
#[cfg(test)]
mod patch_catalog_tests;
#[cfg(test)]
mod patch_support_tests;
