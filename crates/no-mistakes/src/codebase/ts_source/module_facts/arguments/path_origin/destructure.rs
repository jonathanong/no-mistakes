//! Object destructuring, including defaults that apply only when the key is absent.
use super::patterns::{accept_parameterized, direct_binding, object_member, proven, Supplied};
use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};

impl<'a> Projector<'a, '_> {
    pub(super) fn destructured_origin(
        &mut self,
        id: &IdentifierReference<'a>,
        depth: usize,
    ) -> Option<PathOrigin> {
        let symbol = self.binding_symbol(id)?;
        if !self.immutable_binding(symbol) {
            return None;
        }
        let semantic = self.semantic;
        let node = semantic.scoping().symbol_declaration(symbol);
        let AstKind::VariableDeclarator(declaration) = semantic.nodes().kind(node) else {
            return None;
        };
        let BindingPattern::ObjectPattern(pattern) = &declaration.id else {
            return None;
        };
        if pattern.rest.is_some() || id.span.start < declaration.span.end {
            return None;
        }
        let init = declaration.init.as_ref()?;
        for property in &pattern.properties {
            if property.computed {
                continue;
            }
            let Some((binding, default)) = direct_binding(&property.value) else {
                continue;
            };
            if binding.symbol_id.get() != Some(symbol) {
                continue;
            }
            let Some(key) = crate::ast::property_key_name(&property.key) else {
                return None;
            };
            return self.bound_member(init, default, &key, depth);
        }
        None
    }

    fn bound_member(
        &mut self,
        init: &'a Expression<'a>,
        default: Option<&'a Expression<'a>>,
        key: &str,
        depth: usize,
    ) -> Option<PathOrigin> {
        match self.supplied(init, key, depth + 1) {
            Supplied::Proven(origin) => Some(origin),
            Supplied::Present(value) => {
                let origin = self.origin(value, depth + 1);
                proven(&origin).then_some(origin)
            }
            Supplied::Absent => default.map(|expr| self.origin(expr, depth + 1)),
            Supplied::Unknown => None,
        }
    }

    fn supplied(&mut self, object: &'a Expression<'a>, key: &str, depth: usize) -> Supplied<'a> {
        if depth >= 16 || !self.take() {
            return Supplied::Unknown;
        }
        match object.get_inner_expression() {
            Expression::ObjectExpression(object) => object_member(object, key),
            Expression::Identifier(id) => {
                let Some(init) = self.const_init(id) else {
                    return Supplied::Unknown;
                };
                self.supplied(init, key, depth + 1)
            }
            Expression::AwaitExpression(awaited) => {
                self.supplied(&awaited.argument, key, depth + 1)
            }
            Expression::CallExpression(call) => self.supplied_call(call, key, depth),
            _ => Supplied::Unknown,
        }
    }

    fn supplied_call(
        &mut self,
        call: &CallExpression<'a>,
        key: &str,
        depth: usize,
    ) -> Supplied<'a> {
        let Some((returned, parameterized)) = self.getter_expression(call) else {
            return Supplied::Unknown;
        };
        match self.supplied(returned, key, depth + 1) {
            Supplied::Present(value) => proven_member(parameterized, self.origin(value, depth + 1)),
            Supplied::Proven(origin) => proven_member(parameterized, origin),
            other => other,
        }
    }
}

fn proven_member<'a>(parameterized: bool, origin: PathOrigin) -> Supplied<'a> {
    match accept_parameterized(parameterized, origin) {
        Some(origin) if proven(&origin) => Supplied::Proven(origin),
        _ => Supplied::Unknown,
    }
}
