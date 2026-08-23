use super::bindings::{
    binding_names, import_binding, register_require_binding, require_member_binding,
    require_module_or_promises, Binding,
};
use super::ResourceVisitor;
use oxc_ast::ast::{
    BindingPattern, ImportDeclaration, ImportDeclarationSpecifier, VariableDeclarator,
};
use std::collections::HashMap;

impl<'a> ResourceVisitor<'a> {
    pub(super) fn register_import(&mut self, import: &ImportDeclaration<'_>) {
        let module = import.source.value.as_str();
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            let (local, imported) = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    (s.local.name.as_str(), "default")
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    (s.local.name.as_str(), "*")
                }
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    (s.local.name.as_str(), s.imported.name().as_str())
                }
            };
            self.declare_binding(local, import_binding(module, imported));
        }
    }

    pub(super) fn declare_binding(&mut self, name: &str, binding: Option<Binding>) {
        self.bindings
            .last_mut()
            .expect("resource visitor always has a program scope")
            .insert(name.to_string(), binding);
    }

    fn declare_binding_at(&mut self, index: usize, name: &str, binding: Option<Binding>) {
        self.bindings
            .get_mut(index)
            .expect("resource visitor scope index is valid")
            .insert(name.to_string(), binding);
    }

    pub(super) fn declare_var_binding(&mut self, name: &str, binding: Option<Binding>) {
        let index = self.function_binding_scopes.last().copied().unwrap_or(0);
        self.declare_binding_at(index, name, binding);
    }

    pub(super) fn binding(&self, name: &str) -> Option<Binding> {
        self.bindings
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .flatten()
    }

    pub(super) fn is_shadowed(&self, name: &str) -> bool {
        self.bindings
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .is_some_and(Option::is_none)
    }

    pub(super) fn invalidate_binding(&mut self, name: &str) {
        for scope in self.bindings.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), None);
                return;
            }
        }
        self.declare_binding(name, None);
    }

    pub(super) fn register_require(&mut self, declarator: &VariableDeclarator<'_>) {
        let Some(init) = &declarator.init else { return };
        if self.is_shadowed("require") {
            return;
        }
        if let Some(binding) = require_member_binding(init) {
            if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                self.declare_binding(id.name.as_str(), Some(binding));
                return;
            }
        }
        let Some(module) = require_module_or_promises(init) else {
            return;
        };
        let mut bindings = HashMap::new();
        register_require_binding(&declarator.id, module, &mut bindings);
        for name in binding_names(&declarator.id) {
            self.declare_binding(&name, bindings.remove(&name));
        }
    }

    /// Flatten the visible bindings for helpers that only need the effective
    /// value. A `None` shadow deliberately removes an outer import.
    pub(super) fn current_bindings(&self) -> HashMap<String, Binding> {
        let mut visible = HashMap::new();
        for scope in &self.bindings {
            for (name, binding) in scope {
                match binding {
                    Some(binding) => {
                        visible.insert(name.clone(), *binding);
                    }
                    None => {
                        visible.remove(name);
                    }
                }
            }
        }
        visible
    }
}
