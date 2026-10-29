use super::*;

impl Syntax<'_> {
    pub(super) fn template_literal<'a>(&mut self, value: &oxc_ast::ast::TemplateLiteral<'a>) {
        for quasi in &value.quasis {
            self.visit_template_element(quasi);
        }
        for expression in &value.expressions {
            let previous = self.metadata.bind_expression();
            self.visit_expression(expression);
            self.metadata.restore(previous);
        }
    }

    pub(super) fn recover_constructor(&mut self, value: &oxc_ast::ast::NewExpression<'_>) {
        if self.recover_helpers {
            self.recovery.construct(
                value,
                &helper_recovery::lower::Context {
                    source: self.source,
                    bindings: &self.recovery_bindings,
                    tags: &self.template_binds,
                },
            );
        }
        let previous = self.metadata.enter_call(None, Some(span(value.span)), None);
        walk::walk_new_expression(self, value);
        self.metadata.restore(previous);
    }
    pub(super) fn recover_arrow(&mut self, value: &oxc_ast::ast::ArrowFunctionExpression<'_>) {
        if self.recover_helpers {
            self.recovery.arrow(
                value,
                &helper_recovery::lower::Context {
                    source: self.source,
                    bindings: &self.recovery_bindings,
                    tags: &self.template_binds,
                },
            );
            self.recovery
                .stack
                .push(crate::codebase::dependencies::extract::CallableId(
                    value.span.start,
                ));
        }
        self.opaque_metadata(|this| walk::walk_arrow_function_expression(this, value));
        if self.recover_helpers {
            self.recovery.stack.pop();
        }
    }
    pub(super) fn recover_call(&mut self, call: &CallExpression<'_>, selected: bool) {
        if !self.recover_helpers {
            return;
        }
        let selected_argument = call.arguments.iter().any(|argument| {
            let Some(expression) = argument.as_expression() else {
                return false;
            };
            self.recovery_bindings
                .references_in(span(expression.span()))
                .any(|(_, at)| {
                    self.targets.contains(&at)
                        || self.aliases.contains_key(&at)
                        || self.typed_targets.contains(&at)
                        || self.namespace_targets.contains_key(&at)
                })
        });
        self.recovery.call(
            call,
            &helper_recovery::lower::Context {
                source: self.source,
                bindings: &self.recovery_bindings,
                tags: &self.template_binds,
            },
            selected || selected_argument,
        );
    }

    pub(super) fn recover_function(&mut self, value: &Function<'_>, flags: ScopeFlags) {
        if self.recover_helpers {
            self.recovery.function(
                value,
                &helper_recovery::lower::Context {
                    source: self.source,
                    bindings: &self.recovery_bindings,
                    tags: &self.template_binds,
                },
            );
            self.recovery
                .stack
                .push(crate::codebase::dependencies::extract::CallableId(
                    value.span.start,
                ));
        }
        self.opaque_metadata(|this| walk::walk_function(this, value, flags));
        if self.recover_helpers {
            self.recovery.stack.pop();
        }
    }
}
