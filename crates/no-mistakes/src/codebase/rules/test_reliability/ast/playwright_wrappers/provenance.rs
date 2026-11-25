use super::super::bindings::Bindings;
use super::super::playwright_bindings::ValueTrace;
use super::body::WrapperBody;
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression, FunctionBody, IdentifierReference};
use oxc_ast_visit::Visit;
use oxc_semantic::SymbolId;

impl<'ast> Bindings<'_, 'ast> {
    pub(in crate::codebase::rules::test_reliability::ast) fn non_const_or_destructured_trace(
        &self,
        symbol: SymbolId,
    ) -> ValueTrace {
        let Some(init) = self.any_var_initializer(symbol) else {
            return ValueTrace::None;
        };
        self.unresolved_if_provenance(init, "non-constant or destructured fixture binding")
    }

    pub(in crate::codebase::rules::test_reliability::ast) fn unresolved_if_provenance(
        &self,
        expr: &Expression<'_>,
        reason: &'static str,
    ) -> ValueTrace {
        let mut finder = ProvenanceFinder::new(self);
        finder.visit_expression(expr);
        finder.finish(reason)
    }

    pub(super) fn unresolved_if_body_provenance(
        &self,
        body: &FunctionBody<'_>,
        reason: &'static str,
    ) -> ValueTrace {
        let mut finder = ProvenanceFinder::new(self);
        finder.visit_function_body(body);
        finder.finish(reason)
    }

    pub(super) fn unresolved_if_call_provenance(
        &self,
        call: &CallExpression<'_>,
        reason: &'static str,
    ) -> ValueTrace {
        let mut finder = ProvenanceFinder::new(self);
        finder.visit_call_expression(call);
        finder.finish(reason)
    }
}

struct ProvenanceFinder<'a, 'scope, 'ast> {
    bindings: &'a Bindings<'scope, 'ast>,
    found: bool,
    visited: FxHashSet<SymbolId>,
}

impl<'a, 'scope, 'ast> ProvenanceFinder<'a, 'scope, 'ast> {
    fn new(bindings: &'a Bindings<'scope, 'ast>) -> Self {
        Self {
            bindings,
            found: false,
            visited: FxHashSet::default(),
        }
    }

    fn finish(self, reason: &'static str) -> ValueTrace {
        if self.found {
            ValueTrace::Unresolved(reason)
        } else {
            ValueTrace::None
        }
    }
}

impl<'a> Visit<'a> for ProvenanceFinder<'_, '_, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if self.found {
            return;
        }
        let Some(symbol) = self.bindings.symbol(ident) else {
            return;
        };
        if self.bindings.import_origin(symbol, None).is_some()
            || self.bindings.is_namespace_import(symbol)
        {
            self.found = true;
            return;
        }
        if !self.visited.insert(symbol) {
            return;
        }
        if let Some(init) = self.bindings.any_var_initializer(symbol) {
            self.visit_expression(init);
            return;
        }
        if let Some((_, body)) = self.bindings.function_def(symbol) {
            match body {
                WrapperBody::Expr(expr) => self.visit_expression(expr),
                WrapperBody::Block(block) => self.visit_function_body(block),
            }
        }
    }
}
