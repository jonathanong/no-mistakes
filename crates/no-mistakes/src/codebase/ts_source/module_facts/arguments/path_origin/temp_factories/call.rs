use super::{super::Projector, TempFactoryBinding};
use oxc_ast::ast::*;

impl Projector<'_, '_> {
    pub(in crate::codebase::ts_source::module_facts::arguments::path_origin) fn is_temp_factory(
        &self,
        callee: &Expression<'_>,
    ) -> bool {
        match callee {
            Expression::Identifier(id) => self
                .binding_symbol(id)
                .and_then(|symbol| self.temp_factories.bindings.get(&symbol.index()))
                .is_some_and(|binding| matches!(binding, TempFactoryBinding::Named(_))),
            Expression::StaticMemberExpression(member) => {
                let (root, nested) = match member.object.get_inner_expression() {
                    Expression::StaticMemberExpression(inner)
                        if inner.property.name == "promises" =>
                    {
                        (&inner.object, true)
                    }
                    other => (other, false),
                };
                let (sync, promises_safe) = match root.get_inner_expression() {
                    Expression::Identifier(id) => {
                        let binding = self
                            .binding_symbol(id)
                            .and_then(|symbol| self.temp_factories.bindings.get(&symbol.index()));
                        let Some(TempFactoryBinding::Module {
                            sync,
                            promises_safe,
                        }) = binding
                        else {
                            return false;
                        };
                        (*sync, *promises_safe)
                    }
                    Expression::CallExpression(call) => {
                        let Some(&sync) = self
                            .temp_factories
                            .direct_loads
                            .get(&(call.span.start, call.span.end))
                        else {
                            return false;
                        };
                        if sync && self.temp_factories.unsafe_sync
                            || !sync && self.temp_factories.unsafe_promises
                        {
                            return false;
                        }
                        (sync, !self.temp_factories.unsafe_promises)
                    }
                    _ => return false,
                };
                if nested {
                    sync && promises_safe && member.property.name == "mkdtemp"
                } else {
                    member.property.name == "mkdtemp"
                        || sync && member.property.name == "mkdtempSync"
                }
            }
            _ => false,
        }
    }
}
