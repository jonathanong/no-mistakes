use super::*;
impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn forget_closed(&mut self, name: &str) {
        if let Some(i) = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
        {
            self.literal_objects.closed.remove_at(i, name);
            self.frozen_aliases
                .retain(|alias| alias.scope != i || alias.name != name);
        }
    }

    /// Drops a helper-returned record object after a property write or a call
    /// escape. `Object.hasOwn(record, key)` is not that object.
    pub(in crate::codebase::postgres::embedded::walk) fn invalidate_record_object(
        &mut self,
        name: &str,
    ) {
        let Some(index) = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
        else {
            return;
        };
        let helper_object = self
            .literal_objects
            .closed
            .frames
            .get(index)
            .and_then(|frame| frame.get(name))
            .is_some_and(|arms| {
                arms.iter().any(|arm| {
                    matches!(arm.node, Node::Object(_))
                        && arm
                            .choices
                            .iter()
                            .any(|(choice, _)| choice & super::HELPER_OBJECT_CHOICE != 0)
                })
            });
        let alias = self
            .frozen_aliases
            .iter()
            .any(|alias| alias.scope == index && alias.name == name);
        if helper_object || alias {
            self.forget_closed(name);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn forget_closed_flat_export(
        &mut self,
        name: &str,
        init: &Expression<'_>,
    ) {
        if !matches!(unwrap_ts_wrappers(init), Expression::ObjectExpression(_)) {
            return;
        }
        let Some(arms) = lookup(self, name) else {
            return;
        };
        if !arms.iter().all(|arm|matches!(&arm.node,Node::Object(fields) if fields.values().all(|value|matches!(value,Node::Text(_))))) { return; }
        let roots: Vec<_> = arms.iter().filter_map(|arm| arm.root.as_ref()).collect();
        for frame in &mut self.literal_objects.closed.frames {
            frame.retain(|_, values| {
                !values.iter().any(|value| {
                    value
                        .root
                        .as_ref()
                        .is_some_and(|root| roots.contains(&root))
                })
            });
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn note_closed_imports(
        &mut self,
        program: &Program<'_>,
    ) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            let module = crate::ast::js_literal(&import.source);
            for specifier in import.specifiers.iter().flatten() {
                let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                    continue;
                };
                if named.import_kind.is_type() {
                    continue;
                }
                let export = crate::ast::module_export_name(&named.imported);
                if let Some(entry) = self
                    .executor_options
                    .sql_identifier_assertions
                    .iter()
                    .find(|entry| entry.module == module && entry.name == export)
                {
                    self.literal_objects
                        .closed
                        .assertions
                        .insert(named.local.name.to_string(), entry.clone());
                }
            }
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn note_closed_const(
        &mut self,
        type_annotation: Option<&TSTypeAnnotation<'_>>,
        init: &Expression<'_>,
        scope: usize,
        name: &str,
    ) {
        super::relation::note_const_relation_and_array(self, type_annotation, init, scope, name);
        if let Expression::Identifier(id) = unwrap_ts_wrappers(init) {
            self.invalidate_record_object(id.name.as_str());
        }
        if matches!(unwrap_ts_wrappers(init), Expression::StringLiteral(_)) {
            return;
        }
        super::super::frozen_alias::note(self, init, scope, name);
        let Some(mut arms) = expression(self, init, 8) else {
            return;
        };
        for arm in &mut arms {
            if !matches!(arm.node, Node::Text(_)) && arm.root.is_none() {
                arm.root = Some(format!("closed:{}", init.span().start));
            }
        }
        if let Some(frame) = self.literal_objects.closed.frames.get_mut(scope) {
            frame.insert(name.to_string(), arms);
        }
    }
}
impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn exempt_closed_assertion(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        if call.optional || self.shadowed_locally(id.name.as_str()) {
            return;
        }
        let Some(assertion) = self.literal_objects.closed.assertions.get(id.name.as_str()) else {
            return;
        };
        if let Some(argument) = call
            .arguments
            .get(assertion.allowlist_argument)
            .and_then(Argument::as_expression)
        {
            self.exempt_literal_read(argument);
        }
    }
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn exempt_closed_const_alias(
        &mut self,
        init: &Expression<'_>,
    ) {
        let init = unwrap_ts_wrappers(init);
        match init {
            Expression::Identifier(id)
                if self
                    .literal_objects
                    .closed
                    .has(&self.scopes, id.name.as_str()) =>
            {
                self.exempt_literal_read(init)
            }
            Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_) => {
                self.literal_objects.member_uses.push(init.span())
            }
            _ => {}
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn exempt_closed_set_input(
        &mut self,
        created: &NewExpression<'_>,
    ) {
        if self.set_intrinsic_safe
            && self.lookup("Set").is_none()
            && created.arguments.len() == 1
            && matches!(unwrap_ts_wrappers(&created.callee),Expression::Identifier(id) if id.name=="Set")
        {
            if let Some(argument) = created.arguments[0].as_expression() {
                self.exempt_closed_const_alias(argument);
            }
        }
    }
}
