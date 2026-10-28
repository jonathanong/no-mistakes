use super::{Pending, Span};
use crate::codebase::postgres::query_annotation::heads::{syntax, trusted_append};
use crate::codebase::ts_source::{module_facts::TypeScriptModuleFacts, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression, Program};
use oxc_ast_visit::{walk, Visit};

pub(super) fn pending(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
) -> Vec<Pending> {
    let mut visitor = Caller {
        module,
        executors,
        trusted_tags: trusted_append::import_ids(module),
        pending: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.pending
}

struct Caller<'m> {
    module: &'m TypeScriptModuleFacts,
    executors: &'m FxHashSet<Span>,
    trusted_tags: FxHashSet<usize>,
    pending: Vec<Pending>,
}

impl Caller<'_> {
    fn imported(&self, span: Span) -> Option<(&str, &str)> {
        let mut owners = self.module.bindings.iter().filter(|binding| {
            binding.imported
                && binding.runtime
                && binding.references.iter().any(|reference| {
                    reference.runtime && (reference.span.start, reference.span.end) == span
                })
        });
        let owner = owners.next()?;
        if owners.next().is_some() {
            return None;
        }
        let mut imports = self
            .module
            .imports
            .iter()
            .filter(|import| !import.type_only)
            .flat_map(|import| {
                import.bindings.iter().filter_map(move |binding| {
                    (!binding.type_only
                        && binding.binding_id == owner.id
                        && binding.imported != "*")
                        .then_some((import.specifier.as_str(), binding.imported.as_str()))
                })
            });
        let result = imports.next()?;
        imports.next().is_none().then_some(result)
    }

    fn trusted_payload(&self, value: &Expression<'_>) -> bool {
        if !matches!(
            syntax::head(value),
            syntax::Head::Tag | syntax::Head::AppendTag | syntax::Head::AppendPrefixTag
        ) {
            return false;
        }
        let Some(tag) = trusted_append::tag(value) else {
            return false;
        };
        let mut owners = self.module.bindings.iter().filter(|binding| {
            binding.imported
                && binding.runtime
                && binding.references.iter().any(|reference| {
                    reference.runtime && (reference.span.start, reference.span.end) == tag
                })
        });
        let Some(owner) = owners.next() else {
            return false;
        };
        owners.next().is_none() && self.trusted_tags.contains(&owner.id)
    }

    fn candidate(&self, call: &ast::CallExpression<'_>) -> Option<Pending> {
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
        let (specifier, imported) = self.imported((callee.span.start, callee.span.end))?;
        let callbacks = call
            .arguments
            .iter()
            .map(|arg| inline_executor(arg.as_expression()?, self.executors))
            .collect::<Vec<_>>();
        if !callbacks.iter().any(Option::is_some) {
            return None;
        }
        let payloads = call
            .arguments
            .iter()
            .map(|arg| {
                arg.as_expression()
                    .is_some_and(|arg| self.trusted_payload(arg))
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
