//! Index local processors and scoped bindings using the prepared AST.
use crate::fx::FxHashMap;
use crate::queue::extract_factory_scheduler_bind::{
    Collector as ObjectBindingCollector, ObjectBindings,
};
use crate::queue::extract_helpers::{binding_name, export_name, is_queue_package};
use oxc_ast::ast::*;
use oxc_ast::ast_kind::AstKind;
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use std::cell::Cell;

#[path = "extract_factory_index_model.rs"]
mod model;
use model::{Binding, Index};

pub(in crate::queue) struct QueueIndex<'a> {
    pub processor_spans: Vec<(u32, u32, usize)>,
    pub object_bindings: ObjectBindings<'a>,
}

pub(in crate::queue) fn queue_index<'a>(
    program: &'a Program<'a>,
    roles: &crate::config::v2::schema::QueueFactoryRoles,
) -> QueueIndex<'a> {
    let mut collector = Collector {
        index: Index::default(),
        roles,
        object_bindings: ObjectBindingCollector::default(),
    };
    collector.visit_program(program);
    QueueIndex {
        processor_spans: collector.index.processor_spans(),
        object_bindings: collector.object_bindings.bindings,
    }
}

struct Collector<'a, 'r> {
    index: Index,
    roles: &'r crate::config::v2::schema::QueueFactoryRoles,
    object_bindings: ObjectBindingCollector<'a>,
}

impl<'a> Visit<'a> for Collector<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.object_bindings.enter_node(kind);
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        self.object_bindings.leave_node(kind);
    }

    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.object_bindings.enter_scope(scope_id);
        let next = self.index.scopes.len();
        self.index.scopes.push(FxHashMap::default());
        self.index.parents.push(self.index.scope);
        self.index.var_scopes.push(flags.is_var());
        self.index.scope = next;
    }

    fn leave_scope(&mut self) {
        self.index.scope = self.index.parents[self.index.scope];
    }

    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        for spec in import.specifiers.iter().flatten() {
            if let ImportDeclarationSpecifier::ImportSpecifier(spec) = spec {
                let imported = export_name(&spec.imported);
                let binding = if is_queue_package(crate::ast::js_literal(&import.source).as_ref())
                    && matches!(imported.as_str(), "Worker" | "TestWorker")
                {
                    Binding::Constructor
                } else if let Some(role) = self.roles.worker.iter().find(|r| r.callee == imported) {
                    Binding::Factory(role.processor)
                } else {
                    Binding::Other
                };
                self.index
                    .factories
                    .insert(spec.local.name.to_string(), binding);
                self.index.insert(spec.local.name.as_str(), binding);
            }
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
        self.index.scopes[self.index.scope]
            .entry(id.name.to_string())
            .or_insert(Binding::Other);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        if func.is_function_declaration() {
            if let Some(id) = &func.id {
                self.index
                    .insert(id.name.as_str(), Binding::Callable(func.span));
            }
        }
        walk::walk_function(self, func, flags);
    }

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        if decl.kind == VariableDeclarationKind::Var {
            self.index.hoist_variables(decl);
        }
        let previous = self.index.immutable;
        self.index.immutable = decl.kind == VariableDeclarationKind::Const;
        walk::walk_variable_declaration(self, decl);
        self.index.immutable = previous;
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if self.index.immutable {
            if let (Some(name), Some(init)) = (binding_name(&decl.id), &decl.init) {
                let span = match init.get_inner_expression() {
                    Expression::ArrowFunctionExpression(func) => Some(func.span),
                    Expression::FunctionExpression(func) => Some(func.span),
                    _ => None,
                };
                if let Some(span) = span {
                    self.index.insert(&name, Binding::Callable(span));
                }
            }
        }
        walk::walk_variable_declarator(self, decl);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.index.worker(&call.callee, &call.arguments, false);
        walk::walk_call_expression(self, call);
    }

    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = target {
            self.index
                .writes
                .push((self.index.scope, id.name.to_string()));
        }
        walk::walk_assignment_target(self, target);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        property: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.index
            .writes
            .push((self.index.scope, property.binding.name.to_string()));
        walk::walk_assignment_target_property_identifier(self, property);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        self.index.worker(&expr.callee, &expr.arguments, true);
        walk::walk_new_expression(self, expr);
    }
}
