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
                self.deferred_simple_aliases.push(AggregateAliasCandidate {
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
                self.record_object_member_callable_aliases(local, object, declared_at);
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
        self.insert_callable_alias(CallableAliasBinding {
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

    fn insert_callable_alias(&mut self, binding: CallableAliasBinding) {
        let scope = binding.alias.binding_scope;
        let local = binding.alias.local.clone();
        let index = self.callable_aliases.len();
        self.callable_aliases.push(binding);
        self.insert_callable_alias_index_at(scope, local, index);
    }

    fn indexed_callable_alias(&self, scope: usize, local: &str) -> Option<&CallableAlias> {
        let index = self.callable_alias_index_at(scope, local)?;
        self.callable_aliases
            .get(index)
            .map(|binding| &binding.alias)
    }

    fn record_reassigned_callable_alias(&mut self, name: &str, offset: u32) {
        self.record_reassigned_callable_alias_path(name, offset, None, offset);
    }
    fn record_reassigned_callable_alias_path(
        &mut self,
        name: &str,
        offset: u32,
        path: Option<&[Option<String>]>,
        completed_at: u32,
    ) {
        let binding_name = name.split_once('.').map_or(name, |(binding, _)| binding);
        let Some((lexical_scope_depth, binding_scope)) = self
            .local_stack
            .iter()
            .rposition(|scope| scope.contains(binding_name))
            .map(|depth| (depth, self.lexical_scope_ids[depth]))
        else {
            if self.collect_value_flow
                && binding_name == "arguments"
                && self.project_arguments_member(offset, path, completed_at)
            {
                return;
            }
            if self.collect_value_flow
                && (self.imported_bindings.contains(binding_name)
                    || self.predeclared_imported_bindings.contains(binding_name))
            {
                self.invalidate_value_write_with_path(0, name, offset, path, completed_at);
            }
            return;
        };
        self.insert_reassigned_callable_at(binding_scope, name.to_string());
        self.invalidate_value_write_with_path(binding_scope, name, offset, path, completed_at);
        if let Some(index) = self.callable_alias_index_at(binding_scope, name) {
            if let Some(alias) = self.callable_aliases.get_mut(index) {
                if alias.lexical_scope_depth == lexical_scope_depth {
                    alias.alias.invalidated_at.get_or_insert(offset);
                }
            }
        }
    }

    /// `arguments[0].client.get = …` writes the formal at that index. A missing
    /// formal gets one synthetic slot so the call argument is still a receiver.
    fn project_arguments_member(
        &mut self,
        offset: u32,
        path: Option<&[Option<String>]>,
        completed_at: u32,
    ) -> bool {
        let Some((index, tail)) = arguments_index(path) else {
            return false;
        };
        let Some(owner) = self.current_function_id() else {
            return false;
        };
        let found = self
            .value_flow
            .parameters
            .iter()
            .find(|parameter| {
                parameter.owner == owner
                    && parameter.index == index
                    && parameter.projection.is_empty()
            })
            .map(|parameter| (parameter.scope, parameter.name.clone()));
        let (scope, name) =
            found.unwrap_or_else(|| self.synthesize_arguments_parameter(owner, index));
        let dotted = std::iter::once(name.as_str())
            .chain(tail.iter().filter_map(|part| part.as_deref()))
            .collect::<Vec<_>>()
            .join(".");
        self.invalidate_value_write_with_path(scope, &dotted, offset, Some(tail), completed_at);
        true
    }

    fn synthesize_arguments_parameter(&mut self, owner: CallableId, index: usize) -> (usize, String) {
        let scope = self.current_lexical_scope_id();
        let name = format!("arguments[{index}]");
        let initialization_order = self.value_flow.parameters.len();
        self.value_parameter_indexes
            .entry(scope)
            .or_default()
            .entry(name.clone())
            .or_default()
            .push(initialization_order);
        self.value_flow
            .parameters
            .push(extract_value_flow::ValueParameter {
                owner,
                index,
                name: name.clone(),
                scope,
                default: extract_value_flow::ValueExpression::Unknown,
                projection: Vec::new(),
                initialization_order,
                default_order: initialization_order,
                default_mutations: None,
                invalidated_at: None,
                mutation_contexts: fx_set(),
                type_reference: None,
            });
        (scope, name)
    }
}

fn arguments_index(path: Option<&[Option<String>]>) -> Option<(usize, &[Option<String>])> {
    let (head, tail) = path.and_then(|path| path.split_first())?;
    let index = head.as_deref().and_then(|text| text.parse().ok())?;
    Some((index, tail))
}
