impl ImportCollector {
    fn push_type_symbol_reference(&mut self, name: String) {
        let binding = name.split_once('.').map_or(name.as_str(), |(binding, _)| binding);
        if self.type_parameter_shadows(binding) {
            return;
        }
        let name = if self.type_binding_shadows(binding) {
            let Some(name) = self.scoped_type_binding_reference(&name, binding) else {
                return;
            };
            name
        } else {
            name
        };
        self.symbol_references.push(FunctionCall {
            caller: self.current_function(),
            callee: name,
        });
    }

    fn scoped_type_binding_reference(&self, name: &str, binding: &str) -> Option<String> {
        let Some(current) = self.current_function() else {
            return Some(name.to_string());
        };
        let nested = format!("{current}/{binding}");
        if self.known_function_scopes.contains(&nested) {
            return Some(
                name.strip_prefix(binding)
                    .map_or(nested.clone(), |suffix| format!("{nested}{suffix}")),
            );
        }
        if self
            .type_local_stack
            .last()
            .is_some_and(|scope| scope.contains(binding))
        {
            return None;
        }
        Some(name.to_string())
    }

    fn add_type_parameter_names(&mut self, params: Option<&TSTypeParameterDeclaration<'_>>) {
        let Some(params) = params else { return };
        if self.type_local_stack.is_empty() {
            self.type_local_stack.push(HashSet::new());
        }
        if self.type_parameter_stack.is_empty() {
            self.type_parameter_stack.push(HashSet::new());
        }
        for param in &params.params {
            if let Some(scope) = self.type_parameter_stack.last_mut() {
                scope.insert(param.name.name.to_string());
            }
            if let Some(scope) = self.type_local_stack.last_mut() {
                scope.insert(param.name.name.to_string());
            }
        }
    }

    fn type_parameter_shadows(&self, name: &str) -> bool {
        self.type_parameter_stack
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn type_binding_shadows(&self, name: &str) -> bool {
        self.type_local_stack
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
}

fn record_statement_type_binding(collector: &mut ImportCollector, statement: &Statement<'_>) {
    match statement {
        Statement::TSTypeAliasDeclaration(declaration) => {
            collector.add_type_binding_name(declaration.id.name.as_str());
        }
        Statement::TSInterfaceDeclaration(declaration) => {
            collector.add_type_binding_name(declaration.id.name.as_str());
        }
        _ => {}
    }
}

fn visit_ts_type_reference_without_name_walk<'a>(
    collector: &mut ImportCollector,
    reference: &TSTypeReference<'a>,
) {
    if let Some(name) = type_reference_name(reference) {
        collector.push_type_symbol_reference(name);
    }
    if let Some(type_arguments) = &reference.type_arguments {
        collector.visit_ts_type_parameter_instantiation(type_arguments);
    }
}

fn visit_ts_type_parameter_without_name_walk<'a>(
    collector: &mut ImportCollector,
    parameter: &TSTypeParameter<'a>,
) {
    collector.add_type_binding_name(parameter.name.name.as_str());
    if let Some(constraint) = &parameter.constraint {
        collector.visit_ts_type(constraint);
    }
    if let Some(default) = &parameter.default {
        collector.visit_ts_type(default);
    }
}
