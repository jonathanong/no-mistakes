use super::*;

pub(super) fn unused(node: NodeId, semantic: &Semantic<'_>) -> bool {
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
                if !matches!(semantic.nodes().kind(current), AstKind::AwaitExpression(_)) {
                    return false;
                }
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
                            flags.is_value() && !flags.is_value_as_type()
                        });
            }
            _ => return false,
        }
    }
}
