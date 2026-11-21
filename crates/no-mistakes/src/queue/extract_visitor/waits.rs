use super::QueueVisitor;
use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, CallExpression, Expression, PropertyKey,
    VariableDeclaration, VariableDeclarationKind,
};

impl<'a> QueueVisitor<'a> {
    pub(super) fn note_wait_call(&mut self, call: &CallExpression<'a>) {
        self.note_rate_limit_call(call);
        if self.record_waits
            && (self.wait_value(&call.callee)
                || call.arguments.iter().any(|arg| {
                    arg.as_expression()
                        .is_some_and(|expression| self.wait_value(expression))
                }))
        {
            super::super::extract_factory::note_wait(self, call);
        }
    }

    pub(super) fn note_wait_aliases(&mut self, decl: &VariableDeclaration<'a>) {
        if !self.record_waits || decl.kind != VariableDeclarationKind::Const {
            return;
        }
        for declarator in &decl.declarations {
            let Some(init) = &declarator.init else {
                continue;
            };
            match &declarator.id {
                BindingPattern::BindingIdentifier(id) if self.wait_value(init) => {
                    self.insert_wait_alias(id, declarator.span.end);
                }
                BindingPattern::ObjectPattern(object) => {
                    for property in &object.properties {
                        if property.computed
                            && !matches!(
                                property.key,
                                PropertyKey::StringLiteral(_) | PropertyKey::TemplateLiteral(_)
                            )
                        {
                            continue;
                        }
                        if crate::ast::property_key_name(&property.key)
                            .is_some_and(|name| self.wait_name(&name))
                        {
                            if let BindingPattern::BindingIdentifier(id) = &property.value {
                                self.insert_wait_alias(id, declarator.span.end);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn note_wait_import(
        &mut self,
        id: &BindingIdentifier<'_>,
        imported: &str,
        end: u32,
    ) {
        if self.record_waits && self.roles.is_wait_wrapper(imported) {
            self.insert_wait_alias(id, end);
        }
    }

    fn wait_name(&self, name: &str) -> bool {
        wait_method(name) || self.roles.is_wait_wrapper(name)
    }

    fn insert_wait_alias(&mut self, id: &BindingIdentifier<'_>, end: u32) {
        let Some((scoping, symbol)) = self.scoping.zip(id.symbol_id.get()) else {
            return;
        };
        if scoping.symbol_redeclarations(symbol).is_empty()
            && scoping
                .get_resolved_references(symbol)
                .all(|reference| !reference.is_write())
        {
            self.wait_aliases.insert(symbol, end);
        }
    }

    fn wait_value(&self, expression: &Expression<'a>) -> bool {
        match expression.get_inner_expression() {
            Expression::StaticMemberExpression(member) => {
                self.wait_name(member.property.name.as_str())
            }
            Expression::ComputedMemberExpression(member) => {
                crate::ast::computed_property_name(member).is_some_and(|name| self.wait_name(&name))
            }
            Expression::Identifier(id) if self.roles.is_wait_wrapper(id.name.as_str()) => true,
            Expression::Identifier(id) => self
                .scoping
                .zip(id.reference_id.get())
                .and_then(|(scoping, reference)| scoping.get_reference(reference).symbol_id())
                .and_then(|symbol| self.wait_aliases.get(&symbol))
                .is_some_and(|end| *end <= id.span.start),
            _ => false,
        }
    }
}

fn wait_method(name: &str) -> bool {
    matches!(name, "addAndWait" | "waitUntilFinished")
}
