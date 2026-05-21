use std::collections::HashSet;

#[derive(Default)]
pub(super) struct ClientScopes {
    stack: Vec<ClientScope>,
}

#[derive(Default)]
struct ClientScope {
    client_names: HashSet<String>,
    client_callee_names: HashSet<String>,
    client_factory_callee_names: HashSet<String>,
    shadowed_names: HashSet<String>,
    tracks_var_bindings: bool,
}

impl ClientScopes {
    pub(super) fn new() -> Self {
        Self {
            stack: vec![ClientScope {
                tracks_var_bindings: true,
                ..Default::default()
            }],
        }
    }

    pub(super) fn enter(&mut self, tracks_var_bindings: bool) {
        self.stack.push(ClientScope {
            tracks_var_bindings,
            ..Default::default()
        });
    }

    pub(super) fn leave(&mut self) {
        self.stack.pop();
    }

    pub(super) fn add_client_name(&mut self, name: String, in_var_declaration: bool) {
        let scope = self.target_scope_mut(in_var_declaration);
        scope.shadowed_names.remove(&name);
        scope.client_names.insert(name);
    }

    pub(super) fn add_client_callee_name(&mut self, name: String, in_var_declaration: bool) {
        let scope = self.target_scope_mut(in_var_declaration);
        scope.shadowed_names.remove(&name);
        scope.client_callee_names.insert(name);
    }

    pub(super) fn add_client_factory_callee_name(
        &mut self,
        name: String,
        in_var_declaration: bool,
    ) {
        let scope = self.target_scope_mut(in_var_declaration);
        scope.shadowed_names.remove(&name);
        scope.client_factory_callee_names.insert(name);
    }

    pub(super) fn shadow_name(&mut self, name: String, in_var_declaration: bool) {
        let scope = self.target_scope_mut(in_var_declaration);
        scope.client_names.remove(&name);
        scope.client_callee_names.remove(&name);
        scope.client_factory_callee_names.remove(&name);
        scope.shadowed_names.insert(name);
    }

    pub(super) fn assign_client_name(&mut self, name: String) {
        let scope = self.resolved_scope_mut(&name);
        scope.shadowed_names.remove(&name);
        scope.client_names.insert(name);
    }

    pub(super) fn assign_client_callee_name(&mut self, name: String) {
        let scope = self.resolved_scope_mut(&name);
        scope.shadowed_names.remove(&name);
        scope.client_callee_names.insert(name);
    }

    pub(super) fn assign_client_factory_callee_name(&mut self, name: String) {
        let scope = self.resolved_scope_mut(&name);
        scope.shadowed_names.remove(&name);
        scope.client_factory_callee_names.insert(name);
    }

    pub(super) fn assign_shadow_name(&mut self, name: String) {
        let scope = self.resolved_scope_mut(&name);
        scope.client_names.remove(&name);
        scope.client_callee_names.remove(&name);
        scope.client_factory_callee_names.remove(&name);
        scope.shadowed_names.insert(name);
    }

    fn name_present(scope: &ClientScope, name: &str) -> bool {
        scope.client_names.contains(name)
            || scope.client_callee_names.contains(name)
            || scope.client_factory_callee_names.contains(name)
            || scope.shadowed_names.contains(name)
    }

    pub(super) fn is_client_name(&self, name: &str) -> bool {
        self.stack
            .iter()
            .rev()
            .find(|scope| Self::name_present(scope, name))
            .is_some_and(|scope| scope.client_names.contains(name))
    }

    pub(super) fn is_client_callee_name(&self, name: &str) -> bool {
        self.stack
            .iter()
            .rev()
            .find(|scope| Self::name_present(scope, name))
            .is_some_and(|scope| scope.client_callee_names.contains(name))
    }

    pub(super) fn is_client_factory_callee_name(&self, name: &str) -> bool {
        self.stack
            .iter()
            .rev()
            .find(|scope| Self::name_present(scope, name))
            .is_some_and(|scope| scope.client_factory_callee_names.contains(name))
    }

    pub(super) fn is_shadowed_name(&self, name: &str) -> bool {
        self.stack
            .iter()
            .rev()
            .find(|scope| Self::name_present(scope, name))
            .is_some_and(|scope| scope.shadowed_names.contains(name))
    }

    fn target_scope_mut(&mut self, in_var_declaration: bool) -> &mut ClientScope {
        let var_scope_index = in_var_declaration
            .then(|| {
                self.stack
                    .iter()
                    .rposition(|scope| scope.tracks_var_bindings)
            })
            .flatten();
        if let Some(index) = var_scope_index {
            return &mut self.stack[index];
        }
        self.stack.last_mut().expect("scope stack is never empty")
    }

    fn resolved_scope_mut(&mut self, name: &str) -> &mut ClientScope {
        if let Some(index) = self.stack.iter().rposition(|scope| {
            scope.client_names.contains(name)
                || scope.client_callee_names.contains(name)
                || scope.client_factory_callee_names.contains(name)
                || scope.shadowed_names.contains(name)
        }) {
            return &mut self.stack[index];
        }
        self.stack.last_mut().expect("scope stack is never empty")
    }
}
