use super::{super::Projector, TempFactoryBinding, TempReturn};
use oxc_ast::ast::*;

impl Projector<'_, '_> {
    pub(in crate::codebase::ts_source::module_facts::arguments::path_origin) fn is_temp_factory(
        &self,
        callee: &Expression<'_>,
    ) -> bool {
        self.temp_factory_result(callee).is_some()
    }

    pub(in crate::codebase::ts_source::module_facts::arguments::path_origin) fn temp_factory_result(
        &self,
        callee: &Expression<'_>,
    ) -> Option<TempReturn> {
        match callee {
            Expression::Identifier(id) => self
                .binding_symbol(id)
                .and_then(|symbol| self.temp_factories.bindings.get(&symbol.index()))
                .and_then(super::result::named_result),
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
                            return None;
                        };
                        (*sync, *promises_safe)
                    }
                    Expression::CallExpression(call) => {
                        let &sync = self
                            .temp_factories
                            .direct_loads
                            .get(&(call.span.start, call.span.end))?;
                        if sync && self.temp_factories.unsafe_sync
                            || !sync && self.temp_factories.unsafe_promises
                        {
                            return None;
                        }
                        (sync, !self.temp_factories.unsafe_promises)
                    }
                    _ => return None,
                };
                member_result(member.property.name.as_str(), sync, nested, promises_safe)
            }
            _ => None,
        }
    }
}

fn member_result(name: &str, sync: bool, nested: bool, promises_safe: bool) -> Option<TempReturn> {
    if nested {
        return (sync && promises_safe && name == "mkdtemp").then_some(TempReturn::Promise);
    }
    if name == "mkdtempSync" && sync {
        Some(TempReturn::Directory)
    } else if name == "mkdtemp" {
        Some(if sync {
            TempReturn::Callback
        } else {
            TempReturn::Promise
        })
    } else {
        None
    }
}
