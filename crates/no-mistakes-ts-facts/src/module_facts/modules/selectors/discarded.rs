use super::*;

pub(in super::super) fn unused(node: NodeId, semantic: &Semantic<'_>) -> bool {
    let mut current = node;
    loop {
        let Some((id, kind)) = parent(current, semantic) else {
            return false;
        };
        match kind {
            AstKind::AwaitExpression(_) => current = id,
            AstKind::ExpressionStatement(_) => return true,
            AstKind::UnaryExpression(value) => {
                return value.operator == oxc_syntax::operator::UnaryOperator::Void;
            }
            AstKind::VariableDeclarator(value) => {
                let BindingPattern::BindingIdentifier(binding) = &value.id else {
                    return false;
                };
                let Some(symbol) = binding.symbol_id.get() else {
                    return false;
                };
                let exported = semantic.nodes().ancestors(id).take(2).any(|node| {
                    matches!(
                        node.kind(),
                        AstKind::ExportNamedDeclaration(_) | AstKind::ExportDeclaration(_)
                    )
                });
                return !exported
                    && !semantic
                        .scoping()
                        .get_resolved_references(symbol)
                        .any(|reference| {
                            let flags = reference.flags();
                            flags.is_value()
                                && !flags.is_value_as_type()
                                && !matches!(parent(reference.node_id(), semantic),
                                    Some((_, AstKind::UnaryExpression(value)))
                                    if value.operator == oxc_syntax::operator::UnaryOperator::Void)
                        });
            }
            _ => return false,
        }
    }
}
