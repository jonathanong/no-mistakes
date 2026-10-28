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
    pub(super) fn recover_call(&mut self, call: &CallExpression<'_>) {
        if !self.recover_helpers {
            return;
        }
        self.recovery.call(
            call,
            &helper_recovery::lower::Context {
                source: self.source,
                bindings: &self.recovery_bindings,
                tags: &self.template_binds,
            },
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
