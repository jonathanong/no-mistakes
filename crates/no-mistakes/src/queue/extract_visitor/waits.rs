use super::QueueVisitor;
use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, CallExpression, Expression, PropertyKey,
    VariableDeclaration, VariableDeclarationKind,
};

impl<'a> QueueVisitor<'a> {
    pub(super) fn note_wait_call(&mut self, call: &CallExpression<'a>) {
        if self.record_waits
            && (self.wait_value(&call.callee)
                || matches!(call.callee.get_inner_expression(), Expression::Identifier(id)
                    if self.roles.enqueue_and_wait.iter().any(|name| name == id.name.as_str()))
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
                            .is_some_and(|name| wait_method(&name))
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

    fn insert_wait_alias(&mut self, id: &BindingIdentifier<'a>, end: u32) {
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
                wait_method(member.property.name.as_str())
            }
            Expression::ComputedMemberExpression(member) => {
                crate::ast::computed_property_name(member).is_some_and(|name| wait_method(&name))
            }
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
