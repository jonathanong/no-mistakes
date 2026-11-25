use super::super::bindings::Bindings;
use super::super::playwright_bindings::ValueTrace;
use super::body::WrapperBody;
use crate::codebase::rules::test_reliability::playwright_provenance::ImportRef;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{
    CallExpression, ComputedMemberExpression, Expression, FunctionBody, IdentifierReference,
    StaticMemberExpression,
};
use oxc_ast_visit::Visit;

mod receivers;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

impl<'ast> Bindings<'_, 'ast> {
    pub(in crate::codebase::rules::test_reliability::ast) fn non_const_or_destructured_trace(
        &self,
        symbol: SymbolId,
    ) -> ValueTrace {
        let mut finder = ProvenanceFinder::new(self);
        finder.visit_symbol(symbol);
        finder.finish("non-constant or destructured fixture binding")
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
    candidates: Vec<ImportRef>,
    visited: FxHashSet<(SymbolId, Option<String>)>,
    visited_fields: FxHashSet<Span>,
}

impl<'a, 'scope, 'ast> ProvenanceFinder<'a, 'scope, 'ast> {
    fn new(bindings: &'a Bindings<'scope, 'ast>) -> Self {
        Self {
            bindings,
            candidates: Vec::new(),
            visited: FxHashSet::default(),
            visited_fields: FxHashSet::default(),
        }
    }

    fn finish(self, reason: &'static str) -> ValueTrace {
        ValueTrace::unresolved(reason, self.candidates)
    }

    fn record(&mut self, import: ImportRef) {
        if (import.specifier.as_ref() != "@playwright/test" || import.is_playwright_test())
            && !self.candidates.contains(&import)
        {
            self.candidates.push(import);
        }
    }

    fn visit_symbol(&mut self, symbol: SymbolId) {
        self.visit_symbol_member(symbol, None);
    }

    fn visit_symbol_member(&mut self, symbol: SymbolId, member: Option<&str>) {
        if self.bindings.is_namespace_import(symbol) {
            if let Some(import) = self.bindings.import_origin(symbol, member.or(Some("test"))) {
                self.record(import);
            }
            return;
        }
        if !self.visited.insert((symbol, member.map(str::to_owned))) {
            return;
        }
        if let Some(import) = self.bindings.import_origin(symbol, None) {
            self.record(import);
            return;
        }
        if let Some(class) = self.bindings.class_declaration(symbol) {
            for value in receivers::class_field_values(class, None) {
                if self.visited_fields.insert(value.span()) {
                    self.visit_expression(value);
                }
            }
            return;
        }
        if let Some(init) = self.bindings.any_var_initializer(symbol) {
            self.visit_aliased_expression(init, member);
        }
        for value in self.bindings.possible_binding_values(symbol) {
            self.visit_aliased_expression(value, member);
        }
        if let Some((_, body)) = self.bindings.function_def(symbol) {
            match body {
                WrapperBody::Expr(expr) => self.visit_expression(expr),
                WrapperBody::Block(block) => self.visit_function_body(block),
            }
        }
    }

    fn visit_aliased_expression(&mut self, expr: &Expression<'_>, member: Option<&str>) {
        if member.is_some() {
            if let Expression::Identifier(identifier) = unwrap_ts_wrappers(expr) {
                if let Some(symbol) = self.bindings.symbol(identifier) {
                    self.visit_symbol_member(symbol, member);
                    return;
                }
            }
        }
        self.visit_expression(expr);
    }

    fn visit_member(&mut self, object: &Expression<'_>, property: Option<&str>) {
        let object = unwrap_ts_wrappers(object);
        if let Some((class, instance)) = self.bindings.receiver_class(object) {
            if let Some(instance) = instance {
                if self.visited_fields.insert(instance.span()) {
                    self.visit_arguments(&instance.arguments);
                }
            }
            for value in receivers::class_field_values(class, property) {
                if self.visited_fields.insert(value.span()) {
                    self.visit_expression(value);
                }
            }
            return;
        }
        if let Expression::Identifier(identifier) = object {
            if let Some(symbol) = self.bindings.symbol(identifier) {
                self.visit_symbol_member(symbol, property);
                return;
            }
        }
        self.visit_expression(object);
    }
}

impl<'a> Visit<'a> for ProvenanceFinder<'_, '_, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(symbol) = self.bindings.symbol(ident) {
            self.visit_symbol(symbol);
        }
    }

    fn visit_static_member_expression(&mut self, expr: &StaticMemberExpression<'a>) {
        self.visit_member(&expr.object, Some(expr.property.name.as_str()));
    }

    fn visit_computed_member_expression(&mut self, expr: &ComputedMemberExpression<'a>) {
        let property = match &expr.expression {
            Expression::StringLiteral(name) => name.value.as_str(),
            _ => None,
        };
        self.visit_member(&expr.object, property);
    }
}
