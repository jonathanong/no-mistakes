use super::Projector;
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

impl<'a> Projector<'a, '_> {
    pub(super) fn nested_reads_only(&mut self, object: &Expression<'a>) -> bool {
        let mut object = object.get_inner_expression();
        let mut keys = Vec::new();
        while let Expression::StaticMemberExpression(member) = object {
            if !self.take() {
                return false;
            }
            keys.push(member.property.name.as_str());
            object = member.object.get_inner_expression();
        }
        keys.reverse();
        let Expression::Identifier(id) = object else {
            return false;
        };
        let Some(symbol) = self.binding_symbol(id) else {
            return false;
        };
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return false;
            }
            let mut current = reference.node_id();
            let mut matching = true;
            for key in &keys {
                if !self.take() {
                    return false;
                }
                let span = semantic.nodes().kind(current).span();
                let parent = semantic.nodes().parent_node(current);
                let name = match parent.kind() {
                    AstKind::StaticMemberExpression(member) if member.object.span() == span => {
                        Some(member.property.name.as_str())
                    }
                    AstKind::ComputedMemberExpression(member) if member.object.span() == span => {
                        let Expression::StringLiteral(value) =
                            member.expression.get_inner_expression()
                        else {
                            return false;
                        };
                        value.value.as_str()
                    }
                    _ => return false,
                };
                if name != Some(*key) {
                    matching = false;
                    break;
                }
                current = parent.id();
            }
            if !matching {
                continue;
            }
            let span = semantic.nodes().kind(current).span();
            let consumed_as_object = match semantic.nodes().parent_kind(current) {
                AstKind::StaticMemberExpression(member) => member.object.span() == span,
                AstKind::ComputedMemberExpression(member) => member.object.span() == span,
                _ => false,
            };
            if !consumed_as_object {
                return false;
            }
        }
        true
    }
}

impl<'a> Projector<'a, '_> {
    pub(super) fn member_value(
        &mut self,
        mut origin: super::PathOrigin,
        object: &Expression<'a>,
        key: &str,
    ) -> Option<super::PathOrigin> {
        let mut root = object.get_inner_expression();
        let mut path = vec![key];
        while let Expression::StaticMemberExpression(member) = root {
            path.push(member.property.name.as_str());
            root = member.object.get_inner_expression();
        }
        path.reverse();
        let Expression::Identifier(id) = root else {
            return None;
        };
        let mut scalar = false;
        for effect in self.identifier_effects(id, 0, false) {
            scalar |= effect.requirement(&path)?;
        }
        origin.require_scalar |= scalar;
        if scalar
            && origin.value_kind != super::super::PathValueKind::Scalar
            && (origin.value_kind == super::super::PathValueKind::Mutable || origin.param.is_none())
        {
            return None;
        }
        Some(origin)
    }

    fn mutable_uses(&mut self, id: &IdentifierReference<'a>) -> bool {
        let Some(symbol) = self.binding_symbol(id) else {
            return false;
        };
        if self.safe_uses(symbol, 0) {
            return true;
        }
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() {
                return false;
            }
            let declaration = semantic
                .nodes()
                .ancestors(reference.node_id())
                .find_map(|node| match node.kind() {
                    AstKind::VariableDeclarator(declaration) => Some(Some(declaration)),
                    AstKind::ObjectProperty(_)
                    | AstKind::ObjectExpression(_)
                    | AstKind::ArrayExpression(_) => None,
                    _ => Some(None),
                })
                .flatten();
            let Some(declaration) = declaration else {
                return false;
            };
            let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
                return false;
            };
            if !matches!(
                declaration
                    .init
                    .as_ref()
                    .map(Expression::get_inner_expression),
                Some(Expression::ObjectExpression(_) | Expression::ArrayExpression(_))
            ) {
                return false;
            }
            let Some(carrier) = binding.symbol_id.get() else {
                return false;
            };
            if !self.safe_uses(carrier, 1) {
                return false;
            }
        }
        true
    }

    pub(super) fn value_requirement(
        &mut self,
        id: &IdentifierReference<'a>,
        mut origin: super::PathOrigin,
    ) -> super::PathOrigin {
        if origin
            .atoms()
            .iter()
            .any(|atom| atom.value_kind == super::super::PathValueKind::Mutable)
            && !self.mutable_uses(id)
        {
            return super::PathOrigin::unknown();
        }
        if origin.value_kind == super::super::PathValueKind::Unknown && origin.param.is_some() {
            for effect in self.identifier_effects(id, 0, false) {
                origin.require_scalar |= effect.requirement(&[]).unwrap_or(true);
            }
        }
        origin
    }
}
