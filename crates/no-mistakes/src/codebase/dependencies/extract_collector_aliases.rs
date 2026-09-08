impl ImportCollector {
    fn record_const_callable_aliases(&mut self, declaration: &VariableDeclaration<'_>) {
        if declaration.kind != VariableDeclarationKind::Const || !self.is_function_or_module_scope()
        {
            return;
        }
        for declarator in &declaration.declarations {
            let (Some(local), Some(init)) = (
                binding_identifier_name(&declarator.id),
                declarator.init.as_ref(),
            ) else {
                continue;
            };
            let Some(target) = simple_callee_name(init) else {
                continue;
            };
            self.callable_aliases.push(CallableAliasBinding {
                alias: CallableAlias {
                    scope: self.current_function(),
                    local: local.to_string(),
                    target,
                    binding_scope: self.current_lexical_scope_id(),
                },
                lexical_scope_depth: self.local_stack.len() - 1,
            });
        }
    }

    fn is_function_or_module_scope(&self) -> bool {
        match self.function_scope_stack.last().copied() {
            Some(function_scope) => self.local_stack.len() == function_scope + 1,
            // The program frame is lexical as well: a top-level block must not
            // expose a block-local alias as a module alias.
            None => self.local_stack.len() == 1,
        }
    }

    fn record_reassigned_callable_alias(&mut self, name: &str) {
        let binding_name = name.split_once('.').map_or(name, |(binding, _)| binding);
        let Some((lexical_scope_depth, binding_scope)) = self
            .local_stack
            .iter()
            .rposition(|scope| scope.contains(binding_name))
            .map(|depth| (depth, self.lexical_scope_ids[depth]))
        else {
            return;
        };
        if binding_name == name {
            self.reassigned_callable_binding_ids
                .insert((binding_scope, name.to_string()));
        }
        self.reassigned_callable_scopes
            .insert(name.replace('.', "/"));
        let scope = self.current_function();
        self.reassigned_alias_bindings.extend(
            self.callable_aliases
                .iter()
                .filter(|binding| {
                    binding.alias.local == name
                        && binding.alias.scope == scope
                        && binding.lexical_scope_depth == lexical_scope_depth
                })
                .cloned(),
        );
    }
}
