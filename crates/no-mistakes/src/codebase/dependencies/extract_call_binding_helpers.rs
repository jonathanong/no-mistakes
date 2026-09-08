impl ImportCollector {
    fn record_top_level_value_bindings(&mut self, declaration: &VariableDeclaration<'_>) {
        if !self.collect_call_reachability {
            return;
        }
        if self.current_function().is_none() {
            for declarator in &declaration.declarations {
                self.top_level_value_bindings
                    .extend(binding_names(&declarator.id));
            }
        }
    }

    fn record_call_binding_aliases(&mut self, declaration: &VariableDeclaration<'_>) {
        if !self.collect_call_reachability {
            return;
        }
        for declarator in &declaration.declarations {
            self.record_destructured_require_aliases(declarator);
            let Some(local) = binding_identifier_name(&declarator.id) else {
                continue;
            };
            let Some(init) = &declarator.init else {
                continue;
            };
            if let Some((module, export)) = self.call_alias_target(init) {
                self.insert_call_alias(local, module, export);
                continue;
            }
            let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
                continue;
            };
            let Expression::Identifier(callee) = &call.callee else {
                continue;
            };
            if !self.is_require_factory(callee.name.as_str())
                && !self.is_builtin_require_binding(callee.name.as_str())
            {
                continue;
            }
            let Some(specifier) = call.arguments.first().and_then(string_literal_argument) else {
                continue;
            };
            self.insert_call_alias(local, specifier.to_string(), "*".to_string());
        }

        for declarator in &declaration.declarations {
            let Some(local) = binding_identifier_name(&declarator.id) else {
                continue;
            };
            let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
                continue;
            };
            let Expression::Identifier(callee) = &call.callee else {
                continue;
            };
            if self.is_create_require_binding(callee.name.as_str()) {
                self.record_require_factory(local);
            }
        }
    }

    fn record_destructured_require_aliases(&mut self, declarator: &VariableDeclarator<'_>) {
        let BindingPattern::ObjectPattern(pattern) = &declarator.id else {
            return;
        };
        let Some(Expression::CallExpression(call)) = &declarator.init else {
            return;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        if !self.is_require_factory(callee.name.as_str())
            && !self.is_builtin_require_binding(callee.name.as_str())
        {
            return;
        }
        let Some(module) = call.arguments.first().and_then(string_literal_argument) else {
            return;
        };
        for property in &pattern.properties {
            if property.computed {
                continue;
            }
            let Some(export) = crate::codebase::ts_source::static_property_key_name(&property.key)
            else {
                continue;
            };
            let Some(local) = binding_identifier_name(&property.value) else {
                continue;
            };
            self.insert_call_alias(local, module.to_string(), export.to_string());
        }
    }

    fn call_alias_target(&self, expression: &Expression<'_>) -> Option<(String, String)> {
        if let Some(target) = direct_require_member_call(self, expression) {
            return Some(target);
        }
        match expression {
            Expression::Identifier(identifier) => {
                self.visible_call_binding_target(identifier.name.as_str())
            }
            Expression::StaticMemberExpression(member) => {
                let object = simple_callee_name(&member.object)?;
                let (binding, suffix) = object
                    .split_once('.')
                    .map_or((object.as_str(), None), |(binding, suffix)| {
                        (binding, Some(suffix))
                    });
                let (module, imported) = self.visible_call_binding_target(binding)?;
                let member = suffix.map_or_else(
                    || member.property.name.to_string(),
                    |suffix| format!("{suffix}.{}", member.property.name),
                );
                Some((
                    module,
                    if imported == "*" {
                        member
                    } else {
                        format!("{imported}.{member}")
                    },
                ))
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                self.call_alias_target(&parenthesized.expression)
            }
            _ => None,
        }
    }

    fn insert_call_alias(&mut self, local: &str, module: String, export: String) {
        if let Some(aliases) = self.call_alias_stack.last_mut() {
            aliases.insert(local.to_string(), (module, export));
        } else {
            self.call_import_bindings
                .insert(local.to_string(), (module, export));
        }
    }

}
