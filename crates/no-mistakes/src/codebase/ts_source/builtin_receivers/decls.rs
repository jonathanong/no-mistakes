use super::array_escapes;
use super::dates;
use super::known_wrong;
use super::names::{Names, ValueKind};
use super::return_kinds;
use crate::fx::FxHashSet;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) struct Decls<'a> {
    pub(super) prior: Names,
    pub(super) next: Names,
    pub(super) kind: VariableDeclarationKind,
    pub(super) hooks: bool,
    pub(super) called: &'a FxHashSet<String>,
}

impl Decls<'_> {
    fn declare_properties(&mut self, name: &str, ty: &TSType<'_>) {
        for (property, kind) in super::values::property_types(&self.prior, ty) {
            self.next.record_property(name, &property, kind);
        }
    }
}

impl<'a> Visit<'a> for Decls<'_> {
    fn visit_variable_declaration(&mut self, node: &VariableDeclaration<'a>) {
        let outer = std::mem::replace(&mut self.kind, node.kind);
        walk::walk_variable_declaration(self, node);
        self.kind = outer;
    }

    fn visit_variable_declarator(&mut self, node: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(id) = &node.id else {
            return walk::walk_variable_declarator(self, node);
        };
        let annotation = node.type_annotation.as_ref();
        let kind = match (&node.type_annotation, &node.init) {
            (Some(annotation), init) => {
                let kind = self.prior.type_kind(&annotation.type_annotation);
                array_escapes::preserve_empty_array(kind, init.as_ref(), self.kind)
            }
            (None, Some(init)) if self.kind == VariableDeclarationKind::Const => {
                self.prior.value_kind(init, 0)
            }
            _ => None,
        };
        known_wrong::record_custom(
            &mut self.next,
            id.name.as_str(),
            kind,
            annotation.map(|ty| &ty.type_annotation),
            node.init.as_ref(),
        );
        self.next.record(id.name.as_str(), kind);
        if self.kind == VariableDeclarationKind::Const {
            dates::note(
                &mut self.next,
                &self.prior,
                id.name.as_str(),
                annotation.map(|ty| &ty.type_annotation),
                node.init.as_ref(),
            );
        }
        if let Some(annotation) = annotation {
            self.declare_properties(id.name.as_str(), &annotation.type_annotation);
        }
        if let Some(init) = &node.init {
            return_kinds::record_expression(
                &mut self.next,
                &self.prior,
                id.name.as_str(),
                init,
                self.called,
            );
            self.visit_expression(init);
        }
    }

    fn visit_formal_parameter(&mut self, node: &FormalParameter<'a>) {
        let (BindingPattern::BindingIdentifier(id), Some(annotation)) =
            (&node.pattern, &node.type_annotation)
        else {
            return walk::walk_formal_parameter(self, node);
        };
        let kind = self.prior.type_kind(&annotation.type_annotation);
        known_wrong::record_custom(
            &mut self.next,
            id.name.as_str(),
            kind,
            Some(&annotation.type_annotation),
            node.initializer.as_deref(),
        );
        self.next.record(id.name.as_str(), kind);
        dates::note(
            &mut self.next,
            &self.prior,
            id.name.as_str(),
            Some(&annotation.type_annotation),
            node.initializer.as_deref(),
        );
        self.declare_properties(id.name.as_str(), &annotation.type_annotation);
        if let Some(init) = &node.initializer {
            self.visit_expression(init);
        }
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        return_kinds::record_function(&mut self.next, &self.prior, function, self.called);
        walk::walk_function(self, function, flags);
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
        self.next.record(
            node.local.name.as_str(),
            builtin.then_some(ValueKind::Builtin),
        );
        if builtin {
            self.next
                .storage_constructors
                .insert(node.local.name.to_string());
        }
    }

    fn visit_binding_identifier(&mut self, node: &BindingIdentifier<'a>) {
        self.next.record(node.name.as_str(), None);
    }
}
