use super::{Pending, Span, Summary};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax, trusted_append,
};
use crate::codebase::ts_source::{module_facts::TypeScriptModuleFacts, unwrap_ts_wrappers};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{self, Expression, Program};
use oxc_ast_visit::{walk, Visit};
mod append_helpers;
mod index;
mod inline;
mod local_payloads;
use index::Imports;

pub(super) fn project(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    syntax: &syntax::Syntax,
    bindings: &super::super::bindings::Bindings<'_>,
    executors: &FxHashSet<Span>,
    local: &FxHashMap<Span, Summary>,
) -> (Vec<Pending>, Vec<Span>) {
    visit(program, module, executors, local, Some((syntax, bindings)))
}

fn visit<'a, 'm, 'b>(
    program: &'m Program<'a>,
    module: &'m TypeScriptModuleFacts,
    executors: &'m FxHashSet<Span>,
    local: &'m FxHashMap<Span, Summary>,
    source: Option<(&'m syntax::Syntax, &'m Bindings<'b>)>,
) -> (Vec<Pending>, Vec<Span>) {
    let mut visitor = Caller {
        program,
        module,
        executors,
        local,
        source,
        proofs: None,
        imports: None,
        pending: Vec::new(),
        completed: Vec::new(),
    };
    visitor.visit_program(program);
    (visitor.pending, visitor.completed)
}

struct Caller<'a, 'm, 'b> {
    program: &'m Program<'a>,
    module: &'m TypeScriptModuleFacts,
    executors: &'m FxHashSet<Span>,
    local: &'m FxHashMap<Span, Summary>,
    source: Option<(&'m syntax::Syntax, &'m Bindings<'b>)>,
    proofs: Option<(FxHashSet<Span>, FxHashSet<Span>)>,
    imports: Option<Imports<'m>>,
    pending: Vec<Pending>,
    completed: Vec<Span>,
}

impl Caller<'_, '_, '_> {
    fn prepare_proofs(&mut self) {
        if self.proofs.is_some() {
            return;
        }
        let proofs = self
            .source
            .map_or_else(Default::default, |(syntax, bindings)| {
                let helpers =
                    append_helpers::references(self.program, self.module, syntax, bindings);
                let locals = local_payloads::references(syntax, bindings, self.module, &helpers);
                (helpers, locals)
            });
        self.proofs = Some(proofs);
    }

    fn trusted_payload(
        imports: &Imports<'_>,
        trusted_locals: &FxHashSet<Span>,
        value: &Expression<'_>,
    ) -> bool {
        if let Expression::Identifier(id) = unwrap_ts_wrappers(value) {
            return trusted_locals.contains(&(id.span.start, id.span.end));
        }
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

    fn candidate(&mut self, call: &ast::CallExpression<'_>) {
        if call.optional
            || call.arguments.len() > 8
            || call
                .arguments
                .iter()
                .any(|arg| arg.as_expression().is_none())
        {
            return;
        }
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        if !call.arguments.iter().any(|arg| {
            arg.as_expression()
                .is_some_and(|value| inline::may_execute(value, self.executors))
        }) {
            return;
        }
        self.prepare_proofs();
        let (append_helpers, trusted_locals) = self.proofs.as_ref().unwrap();
        let callbacks = call
            .arguments
            .iter()
            .map(|arg| {
                arg.as_expression()
                    .and_then(|value| inline::executor(value, self.executors, append_helpers))
            })
            .collect::<Vec<_>>();
        if !callbacks.iter().any(Option::is_some) {
            return;
        }
        let imports = self
            .imports
            .get_or_insert_with(|| Imports::new(self.module));
        let callee = (callee.span.start, callee.span.end);
        let payloads = call
            .arguments
            .iter()
            .map(|arg| {
                arg.as_expression()
                    .is_some_and(|arg| Self::trusted_payload(imports, trusted_locals, arg))
            })
            .collect();
        if let Some((specifier, imported)) = imports.imported(callee) {
            self.pending.push(Pending {
                specifier: specifier.to_string(),
                imported: imported.to_string(),
                callbacks,
                payloads,
            });
        } else if let Some(summary) = imports
            .local_declaration(callee)
            .and_then(|declaration| self.local.get(&declaration))
        {
            if let Some(span) = super::projected(&callbacks, &payloads, summary) {
                self.completed.push(span);
            }
        }
    }
}

impl<'a, 'm, 'b> Visit<'a> for Caller<'a, 'm, 'b> {
    fn visit_call_expression(&mut self, call: &ast::CallExpression<'a>) {
        self.candidate(call);
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
#[path = "caller/tests.rs"]
mod tests;
