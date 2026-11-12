use super::*;
use oxc_semantic::SymbolId;
#[path = "root_uses.rs"]
mod uses;

struct RootBinding<'a> {
    symbol: SymbolId,
    disposable: bool,
    init: &'a Expression<'a>,
    kind: VariableDeclarationKind,
}

impl<'a> Projector<'a, '_> {
    pub(super) fn callback_root(
        &mut self,
        expression: &Expression<'a>,
        call_node: NodeId,
        callback: SymbolId,
        helper: u32,
    ) -> Option<bool> {
        let root = self.callback_root_binding(expression, call_node, helper)?;
        self.callback_root_factory(&root)?;
        let origin = self.origin(expression, 1);
        if origin.anchor != PathAnchor::Temp
            || origin.value_kind != super::super::PathValueKind::Scalar
            || origin.dynamic
            || !origin.segments.is_empty()
        {
            return None;
        }
        self.callback_root_uses(root.symbol, callback, root.disposable, helper)?;
        Some(root.disposable || self.finally_cleans(call_node, root.symbol, helper))
    }

    fn callback_root_binding(
        &self,
        expression: &Expression<'a>,
        call_node: NodeId,
        helper: u32,
    ) -> Option<RootBinding<'a>> {
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
        if !matches!(declaration.id, BindingPattern::BindingIdentifier(_)) {
            return None;
        }
        let declaration_list = nodes.parent_id(declaration_node);
        let list = nodes.kind(declaration_list).as_variable_declaration()?;
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
        Some(RootBinding {
            symbol,
            disposable,
            init,
            kind: list.kind,
        })
    }

    fn callback_root_factory(&mut self, root: &RootBinding<'a>) -> Option<()> {
        let mut value = root.init.get_inner_expression();
        let awaited = if let Expression::AwaitExpression(inner) = value {
            value = inner.argument.get_inner_expression();
            true
        } else {
            false
        };
        let Expression::CallExpression(factory) = value else {
            return None;
        };
        if root.disposable {
            let kind = if awaited {
                VariableDeclarationKind::AwaitUsing
            } else {
                VariableDeclarationKind::Using
            };
            if root.kind != kind || !self.disposable_factory(factory, awaited) {
                return None;
            }
        } else if root.kind != VariableDeclarationKind::Const
            || !self.is_temp_factory(&factory.callee)
        {
            return None;
        }
        Some(())
    }
}
