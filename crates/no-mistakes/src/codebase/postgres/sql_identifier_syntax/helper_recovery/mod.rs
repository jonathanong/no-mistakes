//! Bounded low-only helper facts, collected in the existing parsed-program walk.
mod application;
mod budget;
pub(crate) use application::ApplicationKey;
mod catalog;
mod construction;
pub(super) mod iterator;
mod locals;
mod project;
mod transports;
mod validation;
pub(crate) use project::project;
mod recovered;
mod sql_text;
mod state;
mod value;
mod values;
pub(crate) use recovered::{ExecutionSite, Recovered};
pub(crate) mod lower;
use super::{span, Span};
use crate::codebase::dependencies::extract::CallableId;
use crate::fx::FxHashMap;
use oxc_ast::ast::{Argument, CallExpression, Function};

#[derive(Clone)]
pub(crate) struct FunctionModel {
    pub id: CallableId,
    pub span: Span,
    pub binding: Option<usize>,
    pub lowered: lower::Lowered,
}
#[derive(Clone)]
pub(crate) struct CallModel {
    pub span: Span,
    pub construction: bool,
    pub callee: Option<usize>,
    pub members: Vec<String>,
    pub arguments: Vec<lower::Lowered>,
    pub caller: Option<CallableId>,
}
#[derive(Clone)]
pub(crate) struct LocalModel {
    pub declaration: Span,
    pub initializer: Span,
    pub lowered: lower::Lowered,
}
#[derive(Default)]
pub(crate) struct FileModels {
    pub dynamic_scope: bool,
    pub iterators: FxHashMap<
        CallableId,
        Vec<
            crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql,
        >,
    >,
    pub stack: Vec<CallableId>,
    pub binding_functions: FxHashMap<usize, Option<CallableId>>,
    pub argument_owners: FxHashMap<Span, usize>,
    pub functions: FxHashMap<CallableId, FunctionModel>,
    pub calls: Vec<CallModel>,
    pub locals: FxHashMap<usize, LocalModel>,
    pub executors:
        std::collections::BTreeMap<super::EmbeddedSqlOptions, crate::fx::FxHashSet<usize>>,
    pub sinks: std::collections::BTreeMap<super::EmbeddedSqlOptions, crate::fx::FxHashSet<Span>>,
    pub writes: crate::fx::FxHashSet<usize>,
}
impl FileModels {
    pub fn local(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclaration<'_>,
        ctx: &lower::Context<'_>,
    ) {
        locals::collect(self, declaration, ctx);
    }
    pub fn arrow(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'_>,
        ctx: &lower::Context<'_>,
    ) {
        if let Some(lowered) = lower::arrow(function, ctx) {
            let id = CallableId(function.span.start);
            self.functions.insert(
                id,
                FunctionModel {
                    id,
                    span: span(function.span),
                    binding: None,
                    lowered,
                },
            );
        }
    }
    pub fn function(&mut self, function: &Function<'_>, ctx: &lower::Context<'_>) {
        let Some(lowered) = lower::function(function, ctx) else {
            return;
        };
        let id = CallableId(function.span.start);
        self.functions.insert(
            id,
            FunctionModel {
                id,
                span: span(function.span),
                binding: function.id.as_ref().and_then(|id| ctx.bindings.id(id.span)),
                lowered,
            },
        );
    }
    pub fn call(&mut self, call: &CallExpression<'_>, ctx: &lower::Context<'_>) {
        if super::chain(&call.callee)
            .is_some_and(|(_, parts)| parts.last().is_some_and(|part| part == "eval"))
        {
            self.dynamic_scope = true;
        }
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
            construction: false,
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
#[path = "tests/abrupt.rs"]
mod abrupt_tests;
#[cfg(test)]
#[path = "tests/locals.rs"]
mod locals_tests;
#[cfg(test)]
mod patch_budget_tests;
#[cfg(test)]
mod patch_catalog_tests;
#[cfg(test)]
mod patch_support_tests;
#[cfg(test)]
mod predicate_tests;
#[cfg(test)]
mod switch_tests;
