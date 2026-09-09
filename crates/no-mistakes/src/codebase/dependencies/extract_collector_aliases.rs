impl ImportCollector {
    fn record_const_callable_aliases(&mut self, declaration: &VariableDeclaration<'_>) {
        if declaration.kind != VariableDeclarationKind::Const {
            return;
        }
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            let declared_at = declarator.span.start;
            self.record_const_aggregate_alias_candidate(&declarator.id, init, declared_at);
            self.record_callable_alias_from_pattern(&declarator.id, init, declared_at);
        }
    }

    fn record_const_aggregate_alias_candidate(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        declared_at: u32,
    ) {
        let (Some(local), Some(target)) =
            (binding_identifier_name(pattern), simple_callee_name(init))
        else {
            return;
        };
        self.aggregate_alias_candidates
            .push(AggregateAliasCandidate {
                binding_scope: self.current_lexical_scope_id(),
                lexical_scope_depth: self.local_stack.len() - 1,
                local: local.to_string(),
                target,
                declared_at,
                owner: self.current_function(),
                owner_id: self.current_function_id(),
            });
    }

    fn record_callable_alias_from_pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        declared_at: u32,
    ) {
        if let Some(local) = binding_identifier_name(pattern) {
            if let Some(target) = self.callable_alias_target(init) {
                self.push_callable_alias(local.to_string(), target, declared_at);
            } else if let Some(target) = simple_callee_name(init) {
                self.deferred_simple_aliases
                    .push(AggregateAliasCandidate {
                        binding_scope: self.current_lexical_scope_id(),
                        lexical_scope_depth: self.local_stack.len() - 1,
                        local: local.to_string(),
                        target,
                        declared_at,
                        owner: self.current_function(),
                        owner_id: self.current_function_id(),
                    });
            }
            if let Expression::ObjectExpression(object) = init {
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        continue;
                    };
                    let Some(member) =
                        crate::codebase::ts_source::static_property_key_name(&property.key)
                    else {
                        continue;
                    };
                    let Some(target) = self.callable_alias_target(&property.value) else {
                        continue;
                    };
                    self.push_callable_alias(format!("{local}.{member}"), target, declared_at);
                }
            }
            return;
        }

        match (pattern, init) {
            (BindingPattern::ObjectPattern(pattern), Expression::ObjectExpression(object)) => {
                for property in &pattern.properties {
                    if property.computed {
                        continue;
                    }
                    let Some(key) =
                        crate::codebase::ts_source::static_property_key_name(&property.key)
                    else {
                        continue;
                    };
                    let Some(value) = object.properties.iter().find_map(|candidate| {
                        let ObjectPropertyKind::ObjectProperty(candidate) = candidate else {
                            return None;
                        };
                        (crate::codebase::ts_source::static_property_key_name(&candidate.key)
                            == Some(key))
                        .then_some(&candidate.value)
                    }) else {
                        continue;
                    };
                    self.record_callable_alias_from_pattern(&property.value, value, declared_at);
                }
            }
            (BindingPattern::ArrayPattern(pattern), Expression::ArrayExpression(array)) => {
                for (pattern, value) in pattern.elements.iter().zip(&array.elements) {
                    let (Some(pattern), Some(value)) = (pattern, value.as_expression()) else {
                        continue;
                    };
                    self.record_callable_alias_from_pattern(pattern, value, declared_at);
                }
            }
            (BindingPattern::AssignmentPattern(pattern), init) => {
                self.record_callable_alias_from_pattern(&pattern.left, init, declared_at);
            }
            _ => {}
        }
    }

    fn callable_alias_target(&self, init: &Expression<'_>) -> Option<String> {
        simple_callee_name(init).filter(|target| {
            matches!(
                self.call_target_identity(target),
                CallTargetIdentity::RepositoryFunction | CallTargetIdentity::ModuleExport
            )
        })
    }

    fn push_callable_alias(&mut self, local: String, target: String, declared_at: u32) {
        self.callable_aliases.push(CallableAliasBinding {
            alias: CallableAlias {
                scope: self.current_function(),
                scope_id: self.current_function_id(),
                local,
                target,
                binding_scope: self.current_lexical_scope_id(),
                declared_at,
                invalidated_at: None,
            },
            lexical_scope_depth: self.local_stack.len() - 1,
        });
    }

    fn record_reassigned_callable_alias(&mut self, name: &str, offset: u32) {
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
        for alias in &mut self.callable_aliases {
            if alias.alias.local == name
                && alias.alias.binding_scope == binding_scope
                && alias.lexical_scope_depth == lexical_scope_depth
            {
                alias.alias.invalidated_at.get_or_insert(offset);
            }
        }
    }
}
