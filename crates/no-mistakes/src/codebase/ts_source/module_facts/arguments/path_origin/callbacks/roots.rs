use super::*;
use oxc_semantic::SymbolId;

impl<'a> Projector<'a, '_> {
    pub(super) fn callback_root(
        &mut self,
        expression: &Expression<'a>,
        call_node: NodeId,
        callback: SymbolId,
        helper: u32,
    ) -> Option<bool> {
        let (identifier, disposable) = match expression.get_inner_expression() {
            Expression::Identifier(identifier) => (identifier.as_ref(), false),
            Expression::StaticMemberExpression(member) if member.property.name == "path" => {
                let Expression::Identifier(identifier) = member.object.get_inner_expression()
                else {
                    return None;
                };
                (identifier.as_ref(), true)
            }
            _ => return None,
        };
        let semantic = self.semantic;
        let nodes = semantic.nodes();
        let symbol = self.binding_symbol(identifier)?;
        let declaration_node = semantic.scoping().symbol_declaration(symbol);
        let AstKind::VariableDeclarator(declaration) = nodes.kind(declaration_node) else {
            return None;
        };
        if !matches!(declaration.id, BindingPattern::BindingIdentifier(_))
            || self.exported.contains(&symbol.index())
        {
            return None;
        }
        let declaration_list = nodes.parent_id(declaration_node);
        let AstKind::VariableDeclaration(list) = nodes.kind(declaration_list) else {
            return None;
        };
        let body = nodes.parent_id(declaration_list);
        if !matches!(nodes.kind(body), AstKind::FunctionBody(_))
            || nodes.parent_kind(body).span().start != helper
        {
            return None;
        }
        let init = declaration.init.as_ref()?;
        if init.span().end >= nodes.kind(call_node).span().start {
            return None;
        }
        let mut value = init.get_inner_expression();
        let awaited = if let Expression::AwaitExpression(inner) = value {
            value = inner.argument.get_inner_expression();
            true
        } else {
            false
        };
        let Expression::CallExpression(factory) = value else {
            return None;
        };
        if disposable {
            let kind = if awaited {
                VariableDeclarationKind::AwaitUsing
            } else {
                VariableDeclarationKind::Using
            };
            if list.kind != kind || !self.disposable_factory(factory, awaited) {
                return None;
            }
        } else if list.kind != VariableDeclarationKind::Const
            || !self.is_temp_factory(&factory.callee)
        {
            return None;
        }
        let origin = self.origin(expression, 1);
        if origin.anchor != PathAnchor::Temp
            || origin.value_kind != super::super::PathValueKind::Scalar
            || origin.dynamic
            || !origin.segments.is_empty()
        {
            return None;
        }
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.is_write() {
                return None;
            }
            if reference.is_type() {
                continue;
            }
            let node = reference.node_id();
            let enclosing = nodes.ancestors(node).find(|ancestor| {
                matches!(
                    ancestor.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            })?;
            if enclosing.kind().span().start != helper {
                return None;
            }
            let mut argument_node = node;
            if disposable {
                argument_node = nodes.parent_id(node);
                if !matches!(nodes.kind(argument_node), AstKind::StaticMemberExpression(member) if member.property.name == "path")
                {
                    return None;
                }
            }
            let invocation_node = nodes.parent_id(argument_node);
            let AstKind::CallExpression(invocation) = nodes.kind(invocation_node) else {
                return None;
            };
            let callback_call = matches!(invocation.callee.get_inner_expression(), Expression::Identifier(id) if self.binding_symbol(id) == Some(callback));
            if !callback_call
                && (disposable || !self.cleanup_call(invocation, symbol, invocation_node))
            {
                return None;
            }
            if callback_call
                && invocation
                    .arguments
                    .iter()
                    .filter(|argument| argument.span() == nodes.kind(argument_node).span())
                    .count()
                    != 1
            {
                return None;
            }
        }
        Some(disposable || self.finally_cleans(call_node, symbol, helper))
    }
}
