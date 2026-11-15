use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{self, Expression};
use oxc_ast_visit::{walk, Visit};

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
        bindings: 0,
        valid: true,
        writing: false,
        in_helper: false,
    };
    finder.visit_program(module);
    finder.valid && finder.bindings == 1
}

struct ObjectBinding {
    found: bool,
}

struct References<'a> {
    record: &'a str,
    allowed: Vec<u32>,
    bindings: u32,
    valid: bool,
    writing: bool,
    in_helper: bool,
}

impl<'a> Visit<'a> for ObjectBinding {
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        self.found |= id.name == "Object";
    }
    fn visit_ts_type(&mut self, _: &ast::TSType<'a>) {}
}

impl<'a> Visit<'a> for References<'a> {
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        if id.name == self.record {
            self.bindings += 1;
        }
    }
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        if id.name == self.record && !self.allowed.contains(&id.span.start) {
            self.valid = false;
        }
    }
    fn visit_simple_assignment_target(&mut self, target: &ast::SimpleAssignmentTarget<'a>) {
        let writing = self.writing;
        self.writing = true;
        walk::walk_simple_assignment_target(self, target);
        self.writing = writing;
    }
    fn visit_function(&mut self, func: &ast::Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let previous = self.in_helper;
        self.in_helper = helper_returns(func, self.record);
        walk::walk_function(self, func, flags);
        self.in_helper = previous;
    }
    fn visit_computed_member_expression(&mut self, member: &ast::ComputedMemberExpression<'a>) {
        if self.in_helper
            && !self.writing
            && !member.optional
            && matches!(
                unwrap_ts_wrappers(&member.expression),
                Expression::Identifier(_)
            )
        {
            if let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) {
                if object.name == self.record {
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
    fn visit_ts_type(&mut self, _: &ast::TSType<'a>) {}
}

fn helper_returns(func: &ast::Function<'_>, record: &str) -> bool {
    let Some(body) = func.body.as_ref() else {
        return false;
    };
    let [ast::Statement::IfStatement(guard), ast::Statement::ReturnStatement(returned)] =
        body.statements.as_slice()
    else {
        return false;
    };
    if guard.alternate.is_some() || !super::guard::throws(&guard.consequent) {
        return false;
    }
    let Some((has_record, _)) = super::guard::has_own(&guard.test) else {
        return false;
    };
    let Some((returned_record, _)) = super::guard::returned_record(returned) else {
        return false;
    };
    has_record == record && returned_record == record
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
