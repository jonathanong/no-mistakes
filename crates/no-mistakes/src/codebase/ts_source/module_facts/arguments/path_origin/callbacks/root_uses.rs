use super::*;

impl<'a> Projector<'a, '_> {
    pub(super) fn callback_root_uses(
        &mut self,
        symbol: SymbolId,
        callback: SymbolId,
        disposable: bool,
        helper: u32,
    ) -> Option<()> {
        let semantic = self.semantic;
        let nodes = semantic.nodes();
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
            self.callback_root_use(node, symbol, callback, disposable)?;
        }
        Some(())
    }

    fn callback_root_use(
        &mut self,
        node: NodeId,
        symbol: SymbolId,
        callback: SymbolId,
        disposable: bool,
    ) -> Option<()> {
        let nodes = self.semantic.nodes();
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
        if !callback_call && (disposable || !self.cleanup_call(invocation, symbol, invocation_node))
        {
            return None;
        }
        Some(())
    }
}

#[cfg(test)]
#[path = "root_uses_tests.rs"]
mod tests;
