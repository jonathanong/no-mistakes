use super::*;

impl ImportCollector {
    pub(super) fn note_builtin_const_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
    ) {
        let BindingPattern::BindingIdentifier(identifier) = pattern else {
            return;
        };
        let unwrapped = crate::syntax_helpers::unwrap_ts_wrappers(init);
        let entry = if unwrapped.is_literal()
            || matches!(
                unwrapped,
                Expression::TemplateLiteral(template) if template.expressions.is_empty()
            ) {
            Some(None)
        } else if let Expression::NewExpression(new) = unwrapped {
            simple_callee_name(&new.callee)
                .filter(|ctor| {
                    self.callee_binding_scope(ctor).is_none()
                        && !self.imported_bindings.contains(ctor)
                        && !self.predeclared_imported_bindings.contains(ctor)
                })
                .map(Some)
        } else {
            None
        };
        if let Some(kind) = entry {
            let scope = self
                .callee_binding_scope(identifier.name.as_str())
                .unwrap_or(0);
            self.value_flow
                .builtin_const_bindings
                .insert((scope, identifier.name.to_string()), kind);
        }
    }

    pub(super) fn note_ambient_builtin_const_binding(
        &mut self,
        declarator: &VariableDeclarator<'_>,
    ) {
        use oxc_ast::ast::TSType;
        if let (BindingPattern::BindingIdentifier(identifier), Some(annotation)) =
            (&declarator.id, &declarator.type_annotation)
        {
            if matches!(
                &annotation.type_annotation,
                TSType::TSArrayType(_)
                    | TSType::TSStringKeyword(_)
                    | TSType::TSNumberKeyword(_)
                    | TSType::TSBooleanKeyword(_)
            ) {
                let scope = self
                    .callee_binding_scope(identifier.name.as_str())
                    .unwrap_or(0);
                self.value_flow
                    .builtin_const_bindings
                    .insert((scope, identifier.name.to_string()), None);
            }
        }
    }
}
