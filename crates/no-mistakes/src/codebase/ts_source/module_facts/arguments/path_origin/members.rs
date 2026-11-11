use super::member_mutations::member_assignment_target;
use super::patterns::accept_parameterized;
use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

impl<'a> Projector<'a, '_> {
    pub(super) fn member_origin(
        &mut self,
        object: &Expression<'a>,
        key: &str,
        depth: usize,
    ) -> Option<PathOrigin> {
        if depth >= 16 || !self.take() {
            return None;
        }
        match object.get_inner_expression() {
            Expression::Identifier(id) => {
                let symbol = self.binding_symbol(id)?;
                if !self.member_reads_only(symbol) {
                    return None;
                }
                if let Some(mut origin) = self.param_origin(id) {
                    // Replacing the pattern key would read a sibling property.
                    if origin.param_member.is_some() {
                        return None;
                    }
                    origin.param_member = Some(key.to_owned());
                    return self.member_value(origin, object, key);
                }
                if let Some(origin) = self.rest_member_origin(id, key, depth + 1) {
                    return Some(origin);
                }
                let init = self.const_init(id)?;
                let origin = self.member_origin(init, key, depth + 1)?;
                self.member_value(origin, object, key)
            }
            Expression::ObjectExpression(object) => {
                let mut value = None;
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return None;
                    };
                    if property.computed || property.method || property.kind != PropertyKind::Init {
                        return None;
                    }
                    if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
                        value = Some(&property.value);
                    }
                }
                value.map(|value| self.origin(value, depth + 1))
            }
            Expression::StaticMemberExpression(_) => {
                if !self.nested_reads_only(object) {
                    return None;
                }
                let mut origin = self.origin(object, depth + 1);
                if origin.param.is_some() {
                    let prefix = origin.param_member.as_ref()?;
                    origin.param_member = Some(format!("{prefix}.{key}"));
                    return self.member_value(origin, object, key);
                }
                let value = self.member_initializer(object, key, depth + 1)?;
                let origin = self.origin(value, depth + 1);
                self.member_value(origin, object, key)
            }
            Expression::CallExpression(call) => {
                if let Some(origin) = super::composite::disposable_path(call, key) {
                    return Some(origin);
                }
                let (returned, parameterized) = self.getter_expression(call)?;
                let origin = self.member_origin(returned, key, depth + 1)?;
                accept_parameterized(parameterized, origin)
            }
            Expression::AwaitExpression(awaited) => {
                self.member_origin(&awaited.argument, key, depth + 1)
            }
            _ => None,
        }
    }

    pub(super) fn member_reads_only(&mut self, symbol: oxc_semantic::SymbolId) -> bool {
        self.member_uses_stable(symbol, None)
    }

    /// The object may be read by this destructure. Every other use must stay a
    /// non-mutating member read, so a property write cannot hide behind a default.
    pub(super) fn destructure_source_stable(&mut self, id: &IdentifierReference<'_>) -> bool {
        let Some(symbol) = self.binding_symbol(id) else {
            return false;
        };
        self.member_uses_stable(symbol, Some(id.span))
    }

    fn member_uses_stable(
        &mut self,
        symbol: oxc_semantic::SymbolId,
        allow: Option<oxc_span::Span>,
    ) -> bool {
        if self.exported.contains(&symbol.index()) {
            return false;
        }
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return false;
            }
            let node = reference.node_id();
            if allow.is_some_and(|span| semantic.nodes().kind(node).span() == span) {
                continue;
            }
            let member = semantic.nodes().parent_id(node);
            if !matches!(
                semantic.nodes().kind(member),
                AstKind::StaticMemberExpression(_)
            ) {
                return false;
            }
            let span = semantic.nodes().kind(member).span();
            let invoked = match semantic.nodes().parent_kind(member) {
                AstKind::CallExpression(call) => call.callee.span() == span,
                AstKind::NewExpression(call) => call.callee.span() == span,
                _ => false,
            };
            if invoked {
                return false;
            }
            if member_assignment_target(semantic.nodes(), member)
                || matches!(
                    semantic.nodes().parent_kind(member),
                    AstKind::UpdateExpression(_) | AstKind::UnaryExpression(_)
                )
            {
                return false;
            }
        }
        true
    }
}
