//! Calls whose receiver is provably an ECMAScript or Node built-in value.
//! The proof is per file and name based: a name is built-in only when every
//! declaration of it is. Anything else stays unproven.
mod callback_arguments;
mod contextual;
mod names;
mod proof;
mod tables;
mod values;
use crate::fx::{FxHashMap, FxHashSet};
use names::{Names, ValueKind};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(crate) use tables::GLOBAL_OBJECTS;

/// Start offsets of call expressions whose callee receiver is a provable built-in.
#[derive(Default)]
pub(crate) struct ProvenCalls {
    pub(crate) calls: FxHashSet<u32>,
    pub(crate) callbacks: FxHashMap<(u32, u32), Option<&'static [usize]>>,
}

pub(crate) fn collect(program: &Program<'_>) -> ProvenCalls {
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
        out: ProvenCalls::default(),
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
    fn declare(&mut self, name: &str, kind: Option<ValueKind>) {
        self.next.record(name, kind);
    }

    fn declare_properties(&mut self, name: &str, ty: &TSType<'_>) {
        for (property, kind) in values::property_types(&self.prior, ty) {
            self.next.record_property(name, &property, kind);
        }
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
        let kind = match (&node.type_annotation, &node.init) {
            (Some(annotation), init) => {
                let kind = self.prior.type_kind(&annotation.type_annotation);
                preserve_empty_array(kind, init.as_ref(), self.kind)
            }
            (None, Some(init)) if self.kind == VariableDeclarationKind::Const => {
                self.prior.value_kind(init, 0)
            }
            _ => None,
        };
        self.declare(id.name.as_str(), kind);
        if let Some(annotation) = &node.type_annotation {
            self.declare_properties(id.name.as_str(), &annotation.type_annotation);
        }
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
        self.declare(
            id.name.as_str(),
            self.prior.type_kind(&annotation.type_annotation),
        );
        self.declare_properties(id.name.as_str(), &annotation.type_annotation);
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
        self.declare(
            node.local.name.as_str(),
            builtin.then_some(ValueKind::AsyncStorage),
        );
        if builtin {
            self.next
                .storage_constructors
                .insert(node.local.name.to_string());
        }
    }

    fn visit_binding_identifier(&mut self, node: &BindingIdentifier<'a>) {
        self.declare(node.name.as_str(), None);
    }
}

fn preserve_empty_array(
    kind: Option<ValueKind>,
    init: Option<&Expression<'_>>,
    declaration_kind: VariableDeclarationKind,
) -> Option<ValueKind> {
    let Some(ValueKind::Array { element, .. }) = kind else {
        return kind;
    };
    let empty = declaration_kind == VariableDeclarationKind::Const
        && matches!(init.map(Expression::without_parentheses), Some(Expression::ArrayExpression(array))
            if array.elements.iter().all(|item| matches!(item, ArrayExpressionElement::Elision(_))));
    Some(ValueKind::Array { element, empty })
}

struct Calls<'n> {
    names: &'n Names,
    out: ProvenCalls,
}

impl<'a> Visit<'a> for Calls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            let method = member.property.name.as_str();
            let shadowed_array_from = method == "from"
                && matches!(&member.object, Expression::Identifier(id)
                    if id.name == "Array" && !self.names.global(&id.name));
            if !shadowed_array_from && self.names.builtin(&member.object, Some(method), 0) {
                self.out.calls.insert(call.span.start);
                let key = (call.span.start, call.span.end);
                if let Some(positions) = self.names.callback_positions(&member.object, method) {
                    let positions = callback_arguments::callable_positions(
                        self.names,
                        call,
                        &member.object,
                        method,
                        positions,
                    );
                    self.out.callbacks.insert(key, Some(positions));
                    contextual::collect(
                        self.names,
                        call,
                        &member.object,
                        method,
                        positions,
                        &mut self.out,
                    );
                } else {
                    self.out.callbacks.insert(key, None);
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
#[path = "builtin_receivers_tests.rs"]
mod tests;
