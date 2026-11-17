use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{self, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use std::cell::Cell;

mod scope;
use scope::Scopes;

pub(super) fn shadows_object(module: &ast::Program<'_>) -> bool {
    let mut finder = ObjectBinding { found: false };
    finder.visit_program(module);
    finder.found
}

/// The record is only read as `record[key]` or `Object.hasOwn(record, key)`.
pub(super) fn references_allowed(module: &ast::Program<'_>, record: &str) -> bool {
    let mut finder = References {
        record,
        allowed: Vec::new(),
        scopes: Scopes::default(),
        references: Vec::new(),
        writing: false,
        allowed_return: None,
        class_name: None,
        class_name_seen: false,
    };
    finder.visit_program(module);
    finder.scopes.root_is_unique()
        && finder
            .references
            .iter()
            .all(|&(scope, allowed)| allowed || !finder.scopes.resolves_to_root(scope))
}

struct ObjectBinding {
    found: bool,
}

struct References<'a> {
    record: &'a str,
    allowed: Vec<u32>,
    scopes: Scopes,
    references: Vec<(usize, bool)>,
    writing: bool,
    allowed_return: Option<u32>,
    class_name: Option<u32>,
    class_name_seen: bool,
}

impl<'a> Visit<'a> for ObjectBinding {
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        self.found |= id.name == "Object";
    }
    fn visit_ts_type(&mut self, _: &ast::TSType<'a>) {}
}

impl<'a> Visit<'a> for References<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes.enter(flags);
        if self.class_name_seen {
            self.scopes.bind_declaration(self.scopes.current());
            self.class_name_seen = false;
        }
    }
    fn leave_scope(&mut self) {
        self.scopes.leave();
    }
    fn visit_variable_declaration(&mut self, declaration: &ast::VariableDeclaration<'a>) {
        let scope = if declaration.kind.is_var() {
            self.scopes.var_scope()
        } else {
            self.scopes.current()
        };
        for declarator in &declaration.declarations {
            super::super::super::for_each_bound_identifier(&declarator.id, &mut |id| {
                if id.name == self.record {
                    self.scopes.assign_binding_scope(id.span.start, scope);
                }
            });
        }
        walk::walk_variable_declaration(self, declaration);
    }
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        if id.name == self.record {
            if self.class_name == Some(id.span.start) {
                self.class_name_seen = true;
            } else {
                self.scopes.bind(id.span.start);
            }
        }
    }
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        if id.name == self.record {
            self.references
                .push((self.scopes.current(), self.allowed.contains(&id.span.start)));
        }
    }
    fn visit_simple_assignment_target(&mut self, target: &ast::SimpleAssignmentTarget<'a>) {
        let writing = self.writing;
        self.writing = true;
        walk::walk_simple_assignment_target(self, target);
        self.writing = writing;
    }
    fn visit_function(&mut self, func: &ast::Function<'a>, flags: ScopeFlags) {
        if func.is_function_declaration()
            && func.id.as_ref().is_some_and(|id| id.name == self.record)
        {
            self.scopes.bind_declaration(self.scopes.current());
        }
        let previous = self.allowed_return;
        self.allowed_return = helper_return_start(func, self.record);
        walk::walk_function(self, func, flags);
        self.allowed_return = previous;
    }
    fn visit_class(&mut self, class: &ast::Class<'a>) {
        let previous = self.class_name;
        self.class_name = (class.r#type == ast::ClassType::ClassExpression)
            .then_some(class.id.as_ref())
            .flatten()
            .filter(|id| id.name == self.record)
            .map(|id| id.span.start);
        walk::walk_class(self, class);
        self.class_name = previous;
    }
    fn visit_arrow_function_expression(&mut self, arrow: &ast::ArrowFunctionExpression<'a>) {
        let previous = self.allowed_return;
        self.allowed_return = None;
        walk::walk_arrow_function_expression(self, arrow);
        self.allowed_return = previous;
    }
    fn visit_computed_member_expression(&mut self, member: &ast::ComputedMemberExpression<'a>) {
        if !self.writing && !member.optional {
            if let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) {
                if object.name == self.record && self.allowed_return == Some(object.span.start) {
                    self.allowed.push(object.span.start);
                }
            }
        }
        walk::walk_computed_member_expression(self, member);
    }
    fn visit_call_expression(&mut self, call: &ast::CallExpression<'a>) {
        if let Some(start) = has_own_record(call, self.record) {
            self.allowed.push(start);
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_ts_as_expression(&mut self, expr: &ast::TSAsExpression<'a>) {
        self.visit_expression(&expr.expression);
    }
    fn visit_ts_satisfies_expression(&mut self, expr: &ast::TSSatisfiesExpression<'a>) {
        self.visit_expression(&expr.expression);
    }
    fn visit_ts_type_assertion(&mut self, expr: &ast::TSTypeAssertion<'a>) {
        self.visit_expression(&expr.expression);
    }
    fn visit_ts_interface_declaration(&mut self, _: &ast::TSInterfaceDeclaration<'a>) {}
    fn visit_ts_type_alias_declaration(&mut self, _: &ast::TSTypeAliasDeclaration<'a>) {}
    fn visit_ts_type(&mut self, _: &ast::TSType<'a>) {}
}

fn helper_return_start(func: &ast::Function<'_>, record: &str) -> Option<u32> {
    let body = func.body.as_ref()?;
    let (name, _, start) = super::guard::guarded_record_with_span(body)?;
    (name == record).then_some(start)
}

fn has_own_record(call: &ast::CallExpression<'_>, record: &str) -> Option<u32> {
    if call.optional || call.arguments.len() != 2 {
        return None;
    }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    if member.optional || object.name != "Object" || member.property.name != "hasOwn" {
        return None;
    }
    let Expression::Identifier(target) = unwrap_ts_wrappers(call.arguments[0].as_expression()?)
    else {
        return None;
    };
    (target.name == record).then_some(target.span.start)
}
