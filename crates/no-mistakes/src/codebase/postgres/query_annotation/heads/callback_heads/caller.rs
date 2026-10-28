use super::{Pending, Span};
use crate::codebase::postgres::query_annotation::heads::{syntax, trusted_append};
use crate::codebase::ts_source::{module_facts::TypeScriptModuleFacts, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression, Program};
use oxc_ast_visit::{walk, Visit};
mod index;
use index::Imports;

pub(super) fn pending(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
) -> Vec<Pending> {
    let mut visitor = Caller {
        module,
        executors,
        imports: None,
        pending: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.pending
}

struct Caller<'m> {
    module: &'m TypeScriptModuleFacts,
    executors: &'m FxHashSet<Span>,
    imports: Option<Imports<'m>>,
    pending: Vec<Pending>,
}

impl Caller<'_> {
    fn trusted_payload(imports: &Imports<'_>, value: &Expression<'_>) -> bool {
        if !matches!(
            syntax::head(value),
            syntax::Head::Tag | syntax::Head::AppendTag | syntax::Head::AppendPrefixTag
        ) {
            return false;
        }
        let Some(tag) = trusted_append::tag(value) else {
            return false;
        };
        imports.trusted_tag(tag)
    }

    fn candidate(&mut self, call: &ast::CallExpression<'_>) -> Option<Pending> {
        if call.optional
            || call.arguments.len() > 8
            || call
                .arguments
                .iter()
                .any(|arg| arg.as_expression().is_none())
        {
            return None;
        }
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        let callbacks = call
            .arguments
            .iter()
            .map(|arg| inline_executor(arg.as_expression()?, self.executors))
            .collect::<Vec<_>>();
        if !callbacks.iter().any(Option::is_some) {
            return None;
        }
        let imports = self
            .imports
            .get_or_insert_with(|| Imports::new(self.module));
        let (specifier, imported) = imports.imported((callee.span.start, callee.span.end))?;
        let payloads = call
            .arguments
            .iter()
            .map(|arg| {
                arg.as_expression()
                    .is_some_and(|arg| Self::trusted_payload(imports, arg))
            })
            .collect();
        Some(Pending {
            specifier: specifier.to_string(),
            imported: imported.to_string(),
            callbacks,
            payloads,
        })
    }
}

impl<'a> Visit<'a> for Caller<'_> {
    fn visit_call_expression(&mut self, call: &ast::CallExpression<'a>) {
        if let Some(candidate) = self.candidate(call) {
            self.pending.push(candidate);
        }
        walk::walk_call_expression(self, call);
    }
}

fn inline_executor(value: &Expression<'_>, executors: &FxHashSet<Span>) -> Option<Span> {
    let Expression::ArrowFunctionExpression(arrow) = unwrap_ts_wrappers(value) else {
        return None;
    };
    if arrow.r#async || arrow.params.rest.is_some() || arrow.params.items.len() != 1 {
        return None;
    }
    let param = &arrow.params.items[0];
    if param.initializer.is_some() || !param.decorators.is_empty() {
        return None;
    }
    let BindingPattern::BindingIdentifier(param) = &param.pattern else {
        return None;
    };
    if matches!(param.name.as_str(), "arguments" | "eval") {
        return None;
    }
    let Expression::CallExpression(call) = unwrap_ts_wrappers(arrow.body.as_expression()?) else {
        return None;
    };
    if call.optional || call.arguments.len() != 1 {
        return None;
    }
    let Expression::Identifier(arg) = unwrap_ts_wrappers(call.arguments.first()?.as_expression()?)
    else {
        return None;
    };
    let span = (call.span.start, call.span.end);
    (arg.name == param.name && executors.contains(&span)).then_some(span)
}

#[cfg(test)]
#[path = "caller/tests.rs"]
mod tests;
