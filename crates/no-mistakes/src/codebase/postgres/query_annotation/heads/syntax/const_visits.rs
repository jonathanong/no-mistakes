use super::{head, span, Head, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, VariableDeclaration, VariableDeclarator};
use oxc_span::GetSpan;

impl Syntax {
    pub(super) fn const_declaration(&mut self, value: &VariableDeclaration<'_>) {
        for declaration in &value.declarations {
            self.finite_const(declaration);
            self.namespace_declaration(declaration);
            if self.collect_helper_heads {
                self.record_destructure(declaration);
            }
            if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                if let Some(Expression::ArrowFunctionExpression(arrow)) = self
                    .collect_helper_heads
                    .then(|| declaration.init.as_ref().map(unwrap_ts_wrappers))
                    .flatten()
                {
                    if let Some(summary) = super::super::helper_heads::arrow(arrow) {
                        if let Some(reference) = summary.head.reference() {
                            self.return_sinks.insert(reference);
                        }
                        self.helper_returns.insert(span(binding.span), summary);
                    }
                }
                if self.collect_helper_heads {
                    if let Some(root) = declaration
                        .init
                        .as_ref()
                        .and_then(super::super::returned_params::root)
                    {
                        self.returned_param_roots.insert(span(binding.span), root);
                    }
                    if let Some(Expression::ArrowFunctionExpression(arrow)) =
                        declaration.init.as_ref().map(unwrap_ts_wrappers)
                    {
                        if let Some(summary) = super::super::helper_records::arrow(arrow) {
                            self.record_returns.insert(span(binding.span), summary);
                        }
                    }
                    if let Some(init) = declaration.init.as_ref() {
                        if let Some(reference) = super::super::helper_heads::callee(init)
                            .or_else(|| super::super::helper_records::append_callee(init))
                        {
                            self.record_initializers
                                .insert(span(binding.span), (reference, span(init.span())));
                        }
                    }
                }
                self.append_declaration(binding, declaration.init.as_ref());
                let head = declaration.init.as_ref().map_or(Head::Unproven, head);
                if head.is_constant() {
                    self.constants.insert(span(binding.span), head);
                    if matches!(
                        declaration.init.as_ref().map(unwrap_ts_wrappers),
                        Some(Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
                    ) {
                        self.string_constants.insert(span(binding.span));
                    }
                }
            }
        }
    }
}

impl Syntax {
    fn namespace_declaration(&mut self, declaration: &VariableDeclarator<'_>) {
        if !self.collect_helper_heads || self.current_function.is_some() {
            return;
        }
        let BindingPattern::ObjectPattern(object) = &declaration.id else {
            return;
        };
        if object.rest.is_some() {
            return;
        }
        let Some(Expression::Identifier(namespace)) =
            declaration.init.as_ref().map(unwrap_ts_wrappers)
        else {
            return;
        };
        let mut candidates = Vec::new();
        for property in &object.properties {
            if property.computed {
                return;
            }
            let Some(exported) = crate::ast::property_key_name(&property.key) else {
                return;
            };
            let BindingPattern::BindingIdentifier(local) = &property.value else {
                return;
            };
            candidates.push(super::super::namespace_heads::Candidate {
                local: span(local.span),
                namespace: span(namespace.span),
                exported: exported.into_owned(),
            });
        }
        self.namespace_destructures.extend(candidates);
    }
}

impl Syntax {
    pub(super) fn variable_declaration(&mut self, value: &VariableDeclaration<'_>) {
        if self.collect_helper_heads {
            self.self_appends
                .declaration(value, self.assignments.context);
        }
        if value.kind.is_const() {
            self.const_declaration(value);
        } else {
            self.assignments.declaration(value);
        }
    }
    pub(super) fn self_append_assignment(
        &mut self,
        value: &oxc_ast::ast::AssignmentExpression<'_>,
    ) {
        if self.collect_helper_heads {
            self.self_appends.assignment(
                value,
                self.assignments.context,
                self.assignments.discarded_assignment,
            );
        }
        self.assignments.assignment(value);
    }
}
