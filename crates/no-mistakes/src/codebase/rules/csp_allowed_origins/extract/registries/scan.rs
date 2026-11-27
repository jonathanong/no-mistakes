use super::Binding;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentTarget, BindingIdentifier, BlockStatement, CallExpression,
    CatchClause, Expression, ForInStatement, ForOfStatement, ForStatement, Function,
    IdentifierReference, Program, SpreadElement, StaticBlock, StaticMemberExpression,
    TSModuleBlock,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::BTreeMap;

pub(super) fn remove_unsound(program: &Program<'_>, bindings: &mut BTreeMap<String, Binding>) {
    let mut scan = Scan {
        bindings,
        nested: 0,
    };
    scan.visit_program(program);
}

struct Scan<'a> {
    bindings: &'a mut BTreeMap<String, Binding>,
    /// Scopes below the module. A binding here hides the module const of the same name.
    nested: u32,
}

impl Scan<'_> {
    fn tracked_ident<'a>(&self, expr: &'a Expression<'a>) -> Option<&'a str> {
        let Expression::Identifier(identifier) = unwrap_ts_wrappers(expr) else {
            return None;
        };
        self.bindings
            .contains_key(identifier.name.as_str())
            .then_some(identifier.name.as_str())
    }

    fn taint(&mut self, name: &str) {
        self.bindings.remove(name);
    }

    fn taint_target(&mut self, target: &AssignmentTarget<'_>) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.taint(identifier.name.as_str());
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                if let Some(name) = self.tracked_ident(&member.object) {
                    self.taint(name);
                }
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                if let Some(name) = self.tracked_ident(&member.object) {
                    self.taint(name);
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        // Name lookup cannot tell this binding from the module const.
        if self.nested > 0 {
            self.taint(identifier.name.as_str());
        }
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.nested += 1;
        walk::walk_function(self, function, flags);
        self.nested -= 1;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.nested += 1;
        walk::walk_arrow_function_expression(self, arrow);
        self.nested -= 1;
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.nested += 1;
        walk::walk_catch_clause(self, clause);
        self.nested -= 1;
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.nested += 1;
        walk::walk_block_statement(self, block);
        self.nested -= 1;
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.nested += 1;
        walk::walk_for_statement(self, statement);
        self.nested -= 1;
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        self.nested += 1;
        walk::walk_for_in_statement(self, statement);
        self.nested -= 1;
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.nested += 1;
        walk::walk_for_of_statement(self, statement);
        self.nested -= 1;
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.nested += 1;
        walk::walk_static_block(self, block);
        self.nested -= 1;
    }

    fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
        self.nested += 1;
        walk::walk_ts_module_block(self, block);
        self.nested -= 1;
    }

    fn visit_export_named_declaration(
        &mut self,
        _export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        // `export { name }` is a read of the binding, not a mutation or alias.
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.taint(identifier.name.as_str());
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            walk::walk_call_expression(self, call);
            return;
        };
        let Some(name) = self.tracked_ident(&member.object) else {
            walk::walk_call_expression(self, call);
            return;
        };
        let name = name.to_string();
        if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1 {
            self.taint(&name);
        }
        for argument in &call.arguments {
            self.visit_argument(argument);
        }
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if !member.optional && self.tracked_ident(&member.object).is_some() {
            return;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_spread_element(&mut self, spread: &SpreadElement<'a>) {
        if self.tracked_ident(&spread.argument).is_some() {
            return;
        }
        walk::walk_spread_element(self, spread);
    }

    fn visit_assignment_expression(&mut self, assignment: &oxc_ast::ast::AssignmentExpression<'a>) {
        self.taint_target(&assignment.left);
        walk::walk_assignment_expression(self, assignment);
    }
}
