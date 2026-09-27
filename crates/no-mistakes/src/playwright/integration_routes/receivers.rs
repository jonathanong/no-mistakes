use crate::playwright::ast;
use oxc_ast::ast::{
    AssignmentExpression, BindingIdentifier, Expression, Program, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::{BTreeMap, BTreeSet};

/// Ambiguous/shadowed receiver names are intentionally rejected rather than
/// conflating lexical bindings. A receiver must have a concrete constructor.
#[derive(Default)]
pub(super) struct Receivers {
    declarations: BTreeMap<String, usize>,
    constructors: BTreeMap<String, String>,
    invalid: BTreeSet<String>,
}

impl Receivers {
    pub(super) fn collect(program: &Program<'_>) -> Self {
        let mut census = Self::default();
        census.visit_program(program);
        census
    }

    pub(super) fn unique(&self, name: &str) -> bool {
        self.declarations.get(name) == Some(&1) && !self.invalid.contains(name)
    }

    pub(super) fn constructor(&self, name: &str) -> Option<&str> {
        self.unique(name)
            .then(|| self.constructors.get(name).map(String::as_str))
            .flatten()
            .filter(|constructor| self.unique(constructor))
    }

    fn assign(&mut self, name: &str, value: Option<&Expression<'_>>) {
        let constructor = value.and_then(|value| match value {
            Expression::NewExpression(new) => ast::expression_path(&new.callee)
                .filter(|path| path.len() == 1)
                .map(|path| path[0].clone()),
            _ => None,
        });
        let Some(constructor) = constructor else {
            if value.is_some() {
                self.invalid.insert(name.to_string());
            }
            return;
        };
        if self
            .constructors
            .get(name)
            .is_some_and(|existing| existing != &constructor)
        {
            self.invalid.insert(name.to_string());
        }
        self.constructors.insert(name.to_string(), constructor);
    }
}

impl<'a> Visit<'a> for Receivers {
    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        *self
            .declarations
            .entry(identifier.name.to_string())
            .or_default() += 1;
    }

    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if let oxc_ast::ast::BindingPattern::BindingIdentifier(binding) = &declaration.id {
            self.assign(binding.name.as_str(), declaration.init.as_ref());
        }
        walk::walk_variable_declarator(self, declaration);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        use oxc_ast::ast::AssignmentTarget;
        match &assignment.left {
            AssignmentTarget::AssignmentTargetIdentifier(binding) => {
                self.assign(binding.name.as_str(), Some(&assignment.right));
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                self.invalidate_object(&member.object)
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                self.invalidate_object(&member.object)
            }
            _ => {}
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

impl Receivers {
    fn invalidate_object(&mut self, object: &Expression<'_>) {
        if let Some(path) = ast::expression_path(object) {
            if let Some(binding) = path.first() {
                self.invalid.insert(binding.clone());
            }
        }
    }
}
