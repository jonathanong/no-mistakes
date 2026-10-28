// Project immutable, statically named CommonJS loads through the existing
// imported-binding path of the canonical call graph.
impl ImportCollector {
    fn is_commonjs_binding(&self, local: &str) -> bool {
        self.callee_binding_scope(local).is_some_and(|scope| {
            self.commonjs_bindings
                .get(&scope)
                .is_some_and(|names| names.contains(local))
                || (self.current_function_id().is_some()
                    && self
                        .predeclared_commonjs_bindings
                        .get(&scope)
                        .is_some_and(|names| names.contains(local)))
        })
    }

    fn record_static_commonjs_bindings(
        &mut self,
        declaration: &VariableDeclaration<'_>,
        predeclare: bool,
    ) {
        if !self.collect_value_flow
            || declaration.kind != VariableDeclarationKind::Const
            || self.current_lexical_scope_id() != 0
            || self.module_block_depth != 0
            || self.local_binding_shadows("require")
        {
            return;
        }
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            let mut expression = init.get_inner_expression();
            let mut members = Vec::new();
            while let Expression::StaticMemberExpression(member) = expression {
                members.push(member.property.name.as_str());
                expression = member.object.get_inner_expression();
            }
            let Expression::CallExpression(call) = expression else {
                continue;
            };
            if call.optional || !is_require_callee(call.callee.get_inner_expression()) {
                continue;
            }
            let Some(specifier) = call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(static_import_specifier)
            else {
                continue;
            };
            members.reverse();
            match &declarator.id {
                BindingPattern::BindingIdentifier(local) => {
                    let (imported, kind) = if members.is_empty() {
                        ("*".to_string(), ImportedBindingKind::Namespace)
                    } else {
                        (members.join("."), ImportedBindingKind::Named)
                    };
                    self.push_commonjs_binding(
                        &specifier,
                        &local.name,
                        &imported,
                        kind,
                        predeclare,
                    );
                }
                BindingPattern::ObjectPattern(object) if members.is_empty() => {
                    for property in &object.properties {
                        let BindingPattern::BindingIdentifier(local) = &property.value else {
                            continue;
                        };
                        if property.computed {
                            continue;
                        }
                        let Some(imported) =
                            crate::codebase::ts_source::static_property_key_name(&property.key)
                        else {
                            continue;
                        };
                        self.push_commonjs_binding(
                            &specifier,
                            &local.name,
                            &imported,
                            ImportedBindingKind::Named,
                            predeclare,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn push_commonjs_binding(
        &mut self,
        specifier: &str,
        local: &str,
        imported: &str,
        kind: ImportedBindingKind,
        predeclare: bool,
    ) {
        if predeclare {
            self.predeclared_commonjs_bindings
                .entry(self.current_lexical_scope_id())
                .or_default()
                .insert(local.to_string());
            self.call_import_bindings.push(ImportedBinding {
                specifier: specifier.to_string(),
                local: local.to_string(),
                imported: imported.to_string(),
                kind,
                is_type_only: false,
            });
        } else {
            self.commonjs_bindings
                .entry(self.current_lexical_scope_id())
                .or_default()
                .insert(local.to_string());
        }
    }
}
