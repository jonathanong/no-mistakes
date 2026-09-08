impl ImportCollector {
    fn record_const_callable_aliases(&mut self, declaration: &VariableDeclaration<'_>) {
        if declaration.kind != VariableDeclarationKind::Const {
            return;
        }
        for declarator in &declaration.declarations {
            let (Some(local), Some(init)) = (
                binding_identifier_name(&declarator.id),
                declarator.init.as_ref(),
            ) else {
                continue;
            };
            if let Some(target) = simple_callee_name(init).filter(|target| {
                matches!(
                    self.call_target_identity(target),
                    CallTargetIdentity::RepositoryFunction | CallTargetIdentity::ModuleExport
                )
            }) {
                self.callable_aliases.push(CallableAliasBinding {
                    alias: CallableAlias {
                        scope: self.current_function(),
                        scope_id: self.current_function_id(),
                        local: local.to_string(),
                        target,
                        binding_scope: self.current_lexical_scope_id(),
                    },
                    lexical_scope_depth: self.local_stack.len() - 1,
                });
            }
            let Expression::ObjectExpression(object) = init else {
                continue;
            };
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                let Some(member) = crate::codebase::ts_source::static_property_key_name(&property.key)
                else {
                    continue;
                };
                let Some(target) = simple_callee_name(&property.value).filter(|target| {
                    matches!(
                        self.call_target_identity(target),
                        CallTargetIdentity::RepositoryFunction | CallTargetIdentity::ModuleExport
                    )
                }) else {
                    continue;
                };
                self.callable_aliases.push(CallableAliasBinding {
                    alias: CallableAlias {
                        scope: self.current_function(),
                        scope_id: self.current_function_id(),
                        local: format!("{local}.{member}"),
                        target,
                        binding_scope: self.current_lexical_scope_id(),
                    },
                    lexical_scope_depth: self.local_stack.len() - 1,
                });
            }
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
        self.reassigned_callable_binding_ids
            .insert((binding_scope, name.to_string()));
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
