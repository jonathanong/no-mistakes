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
                .and_then(|symbol| self.temp_factories.get(&symbol.index()))
                .is_some_and(|binding| matches!(binding, TempFactoryBinding::Named(_))),
            Expression::StaticMemberExpression(member) => {
                let (id, nested) = match member.object.get_inner_expression() {
                    Expression::Identifier(id) => (id, false),
                    Expression::StaticMemberExpression(inner)
                        if inner.property.name == "promises" =>
                    {
                        let Expression::Identifier(id) = inner.object.get_inner_expression() else {
                            return false;
                        };
                        (id, true)
                    }
                    _ => return false,
                };
                self.binding_symbol(id)
                    .and_then(|symbol| self.temp_factories.get(&symbol.index()))
                    .is_some_and(|binding| match binding {
                        TempFactoryBinding::Module {
                            sync,
                            promises_safe,
                        } if nested => *sync && *promises_safe && member.property.name == "mkdtemp",
                        TempFactoryBinding::Module { sync, .. } => {
                            member.property.name == "mkdtemp"
                                || *sync && member.property.name == "mkdtempSync"
                        }
                        TempFactoryBinding::Named(_) => false,
                    })
            }
            _ => false,
        }
    }
}
