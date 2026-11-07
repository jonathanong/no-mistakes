use super::{PathAnchor, PathOrigin, Projector};
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
                    origin.param_member = Some(key.to_owned());
                    return Some(origin);
                }
                let init = self.const_init(id)?;
                self.member_origin(init, key, depth + 1)
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
            Expression::CallExpression(call) => {
                let disposable = matches!(call.callee.get_inner_expression(),
                    Expression::Identifier(id) if id.name == "mkdtempDisposableSync")
                    || matches!(call.callee.get_inner_expression(),
                    Expression::StaticMemberExpression(member) if member.property.name == "mkdtempDisposableSync");
                if key == "path" && disposable {
                    return Some(PathOrigin::anchored(PathAnchor::Temp));
                }
                let returned = self.getter_expression(call)?;
                self.member_origin(returned, key, depth + 1)
            }
            Expression::AwaitExpression(awaited) => {
                self.member_origin(&awaited.argument, key, depth + 1)
            }
            _ => None,
        }
    }

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
        for property in &pattern.properties {
            let BindingPattern::BindingIdentifier(binding) = &property.value else {
                continue;
            };
            if binding.symbol_id.get() == Some(symbol) && !property.computed {
                let key = crate::ast::property_key_name(&property.key)?;
                return self.member_origin(declaration.init.as_ref()?, &key, depth + 1);
            }
        }
        None
    }

    fn member_reads_only(&mut self, symbol: oxc_semantic::SymbolId) -> bool {
        if self.exported.contains(&symbol.index()) {
            return false;
        }
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return false;
            }
            let member = semantic.nodes().parent_id(reference.node_id());
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
            if matches!(
                semantic.nodes().parent_kind(member),
                AstKind::AssignmentExpression(_)
                    | AstKind::UpdateExpression(_)
                    | AstKind::UnaryExpression(_)
            ) {
                return false;
            }
        }
        true
    }
}
