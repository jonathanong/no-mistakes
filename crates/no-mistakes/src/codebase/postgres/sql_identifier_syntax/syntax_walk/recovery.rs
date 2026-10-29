use super::*;

impl Syntax<'_> {
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
