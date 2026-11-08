//! Calls whose receiver is provably an ECMAScript or Node built-in value.
//! The proof is per file and name based: a name is built-in only when every
//! declaration of it is. Anything else stays unproven.
mod names;
mod tables;
use crate::fx::{fx_set, FxHashSet};
use names::{type_builtin, Names};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(crate) use tables::GLOBAL_OBJECTS;

/// Start offsets of call expressions whose callee receiver is a provable built-in.
pub(crate) fn collect(program: &Program<'_>) -> FxHashSet<u32> {
    let mut names = Names::default();
    // Initializer proofs may read names declared later; iterate to settle them.
    for _ in 0..3 {
        let mut decls = Decls {
            prior: names,
            next: Names::default(),
            kind: VariableDeclarationKind::Var,
            hooks: false,
        };
        decls.visit_program(program);
        names = decls.next;
    }
    let mut calls = Calls {
        names: &names,
        out: fx_set(),
    };
    calls.visit_program(program);
    calls.out
}

struct Decls {
    prior: Names,
    next: Names,
    kind: VariableDeclarationKind,
    hooks: bool,
}

impl Decls {
    fn declare(&mut self, name: &str, builtin: bool) {
        self.next
            .map
            .entry(name.to_string())
            .or_default()
            .push(builtin);
    }
}

impl<'a> Visit<'a> for Decls {
    fn visit_variable_declaration(&mut self, node: &VariableDeclaration<'a>) {
        let outer = std::mem::replace(&mut self.kind, node.kind);
        walk::walk_variable_declaration(self, node);
        self.kind = outer;
    }

    fn visit_variable_declarator(&mut self, node: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(id) = &node.id else {
            return walk::walk_variable_declarator(self, node);
        };
        let builtin = match (&node.type_annotation, &node.init) {
            (Some(annotation), _) => type_builtin(&annotation.type_annotation),
            (None, Some(init)) if self.kind == VariableDeclarationKind::Const => {
                self.prior.builtin(init, None, 0)
            }
            _ => false,
        };
        self.declare(id.name.as_str(), builtin);
        if let Some(init) = &node.init {
            self.visit_expression(init);
        }
    }

    fn visit_formal_parameter(&mut self, node: &FormalParameter<'a>) {
        let (BindingPattern::BindingIdentifier(id), Some(annotation)) =
            (&node.pattern, &node.type_annotation)
        else {
            return walk::walk_formal_parameter(self, node);
        };
        self.declare(id.name.as_str(), type_builtin(&annotation.type_annotation));
        if let Some(init) = &node.initializer {
            self.visit_expression(init);
        }
    }

    fn visit_import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        let outer = std::mem::replace(
            &mut self.hooks,
            matches!(
                node.source.value.as_str(),
                Some("async_hooks" | "node:async_hooks")
            ),
        );
        walk::walk_import_declaration(self, node);
        self.hooks = outer;
    }

    fn visit_import_specifier(&mut self, node: &ImportSpecifier<'a>) {
        let builtin = self.hooks && node.imported.name() == "AsyncLocalStorage";
        self.declare(node.local.name.as_str(), builtin);
    }

    fn visit_binding_identifier(&mut self, node: &BindingIdentifier<'a>) {
        self.declare(node.name.as_str(), false);
    }
}

struct Calls<'n> {
    names: &'n Names,
    out: FxHashSet<u32>,
}

impl<'a> Visit<'a> for Calls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            if self
                .names
                .builtin(&member.object, Some(member.property.name.as_str()), 0)
            {
                self.out.insert(call.span.start);
            }
        }
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
#[path = "builtin_receivers_tests.rs"]
mod tests;
