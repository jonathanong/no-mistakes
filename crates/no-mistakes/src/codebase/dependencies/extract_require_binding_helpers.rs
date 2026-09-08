impl ImportCollector {
    fn is_create_require_binding(&self, binding: &str) -> bool {
        matches!(
            self.visible_call_binding_target(binding),
            Some((module, export)) if module == "node:module" && export == "createRequire"
        )
    }

    fn record_require_factory(&mut self, binding: &str) {
        if let Some(factories) = self.require_factory_bindings.last_mut() {
            factories.insert(binding.to_string());
        } else {
            self.top_level_require_factory_bindings
                .insert(binding.to_string());
        }
    }

    fn is_require_factory(&self, binding: &str) -> bool {
        for (locals, factories) in self
            .local_stack
            .iter()
            .rev()
            .zip(self.require_factory_bindings.iter().rev())
        {
            if locals.contains(binding) {
                return factories.contains(binding);
            }
        }
        self.top_level_require_factory_bindings.contains(binding)
    }

    fn is_builtin_require_binding(&self, binding: &str) -> bool {
        binding == "require"
            && !self.local_binding_shadows(binding)
            && !self.top_level_value_bindings.contains(binding)
            && !self.imported_bindings.contains(binding)
    }

    fn should_record_call(&self, callee: &str) -> bool {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        if self.legacy_local_binding_shadows(binding) {
            self.legacy_has_local_function_scope(callee)
        } else {
            true
        }
    }

    fn record_imported_bindings(&mut self, import: &ImportDeclaration<'_>) {
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                    if self.collect_call_reachability
                        && !specifier.import_kind.is_type()
                        && !import.import_kind.is_type()
                    {
                        self.call_import_bindings.insert(
                            specifier.local.name.to_string(),
                            (import.source.value.to_string(), specifier.imported.name().to_string()),
                        );
                    }
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                    if self.collect_call_reachability && !import.import_kind.is_type() {
                        self.call_import_bindings.insert(
                            specifier.local.name.to_string(),
                            (import.source.value.to_string(), "default".to_string()),
                        );
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                    if self.collect_call_reachability && !import.import_kind.is_type() {
                        self.call_import_bindings.insert(
                            specifier.local.name.to_string(),
                            (import.source.value.to_string(), "*".to_string()),
                        );
                    }
                }
            }
        }
    }
}
