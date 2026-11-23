use super::super::selection::closed_property;
use crate::codebase::ts_source::unwrap_ts_wrappers as unwrap;
use crate::fx::FxHashSet;
use crate::queue::extract_helpers::static_string;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::Expression;

#[derive(Default)]
pub(super) struct Origins {
    pub objects: FxHashSet<u32>,
    pub locals: FxHashSet<u32>,
    pub imports: FxHashSet<String>,
    pub unknown: bool,
}

impl Origins {
    pub(super) fn local<'a>(
        &mut self,
        name: &str,
        at: u32,
        bindings: &ObjectBindings<'a>,
        depth: usize,
    ) -> Option<&'a Expression<'a>> {
        if let Some(binding) = bindings.lookup(name, at) {
            self.locals.insert(binding.resolve_at);
            let Some(init) = binding.init else {
                self.unknown = true;
                return None;
            };
            return self.follow(init, bindings, depth + 1);
        }
        self.imports.insert(name.to_string());
        None
    }

    pub(super) fn follow<'a>(
        &mut self,
        expr: &'a Expression<'a>,
        bindings: &ObjectBindings<'a>,
        depth: usize,
    ) -> Option<&'a Expression<'a>> {
        if depth >= 64 {
            self.unknown = true;
            return None;
        }
        let expr = unwrap(expr);
        match expr {
            Expression::Identifier(id) => {
                self.local(id.name.as_str(), id.span.start, bindings, depth)
            }
            Expression::StaticMemberExpression(member) => self.member(
                &member.object,
                Some(member.property.name.as_str()),
                bindings,
                depth,
            ),
            Expression::ComputedMemberExpression(member) => {
                let name = static_string(unwrap(&member.expression));
                self.member(&member.object, name.as_deref(), bindings, depth)
            }
            Expression::ObjectExpression(object) => {
                self.objects.insert(object.span.start);
                Some(expr)
            }
            _ => Some(expr),
        }
    }

    fn member<'a>(
        &mut self,
        container: &'a Expression<'a>,
        name: Option<&str>,
        bindings: &ObjectBindings<'a>,
        depth: usize,
    ) -> Option<&'a Expression<'a>> {
        let container = self.follow(container, bindings, depth + 1);
        let property = match (container, name) {
            (Some(Expression::ObjectExpression(object)), Some(name)) => {
                closed_property(object, name).flatten()
            }
            _ => None,
        };
        let Some(property) = property else {
            self.unknown = true;
            return None;
        };
        self.follow(property, bindings, depth + 1)
    }
}
